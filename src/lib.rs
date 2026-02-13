use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::error::Error as StdError;
use std::fmt;
use std::rc::Rc;

const INTERNAL_RETURN_SLOT: &str = "__bt_internal_return_value__";

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
    readonly: bool,
    required: bool,
}

#[derive(Debug, Clone)]
struct Dom {
    nodes: Vec<Node>,
    root: NodeId,
    id_index: HashMap<String, Vec<NodeId>>,
    active_element: Option<NodeId>,
    active_pseudo_element: Option<NodeId>,
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
            active_element: None,
            active_pseudo_element: None,
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
        let readonly = attrs.contains_key("readonly");
        let required = attrs.contains_key("required");
        let element = Element {
            tag_name,
            attrs,
            value,
            checked,
            disabled,
            readonly,
            required,
        };
        let id = self.create_node(Some(parent), NodeType::Element(element));
        if let Some(id_attr) = self
            .element(id)
            .and_then(|element| element.attrs.get("id").cloned())
        {
            self.index_id(&id_attr, id);
        }
        id
    }

    fn create_detached_element(&mut self, tag_name: String) -> NodeId {
        let element = Element {
            tag_name,
            attrs: HashMap::new(),
            value: String::new(),
            checked: false,
            disabled: false,
            readonly: false,
            required: false,
        };
        self.create_node(None, NodeType::Element(element))
    }

    fn create_detached_text(&mut self, text: String) -> NodeId {
        self.create_node(None, NodeType::Text(text))
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

    fn is_descendant_of(&self, node_id: NodeId, ancestor: NodeId) -> bool {
        let mut cursor = self.parent(node_id);
        while let Some(current) = cursor {
            if current == ancestor {
                return true;
            }
            cursor = self.parent(current);
        }
        false
    }

    fn active_element(&self) -> Option<NodeId> {
        self.active_element
    }

    fn set_active_element(&mut self, node: Option<NodeId>) {
        self.active_element = node;
    }

    fn active_pseudo_element(&self) -> Option<NodeId> {
        self.active_pseudo_element
    }

    fn set_active_pseudo_element(&mut self, node: Option<NodeId>) {
        self.active_pseudo_element = node;
    }

    fn by_id(&self, id: &str) -> Option<NodeId> {
        self.id_index.get(id).and_then(|ids| ids.first().copied())
    }

    fn by_id_all(&self, id: &str) -> Vec<NodeId> {
        self.id_index.get(id).cloned().unwrap_or_default()
    }

    fn index_id(&mut self, id: &str, node_id: NodeId) {
        if id.is_empty() {
            return;
        }
        self.id_index
            .entry(id.to_string())
            .or_default()
            .push(node_id);
    }

    fn unindex_id(&mut self, id: &str, node_id: NodeId) {
        let Some(nodes) = self.id_index.get_mut(id) else {
            return;
        };
        nodes.retain(|candidate| *candidate != node_id);
        if nodes.is_empty() {
            self.id_index.remove(id);
        }
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

    fn inner_html(&self, node_id: NodeId) -> Result<String> {
        if self.element(node_id).is_none() {
            return Err(Error::ScriptRuntime(
                "innerHTML target is not an element".into(),
            ));
        }
        let mut out = String::new();
        for child in &self.nodes[node_id.0].children {
            out.push_str(&self.dump_node(*child));
        }
        Ok(out)
    }

    fn set_inner_html(&mut self, node_id: NodeId, html: &str) -> Result<()> {
        if self.element(node_id).is_none() {
            return Err(Error::ScriptRuntime(
                "innerHTML target is not an element".into(),
            ));
        }

        let ParseOutput { dom: fragment, .. } = parse_html(html)?;

        let old_children = std::mem::take(&mut self.nodes[node_id.0].children);
        for child in old_children {
            self.nodes[child.0].parent = None;
        }

        let children = fragment.nodes[fragment.root.0].children.clone();
        for child in children {
            let _ = self.clone_subtree_from_dom(&fragment, child, Some(node_id))?;
        }

        self.rebuild_id_index();
        Ok(())
    }

    fn insert_adjacent_html(
        &mut self,
        target: NodeId,
        position: InsertAdjacentPosition,
        html: &str,
    ) -> Result<()> {
        let ParseOutput { dom: fragment, .. } = parse_html(html)?;

        let mut children = fragment.nodes[fragment.root.0].children.clone();
        if matches!(position, InsertAdjacentPosition::AfterBegin) {
            children.reverse();
        }

        for child in children {
            let node = self.clone_subtree_from_dom(&fragment, child, None)?;
            self.insert_adjacent_node(target, position, node)?;
        }
        Ok(())
    }

    fn clone_subtree_from_dom(
        &mut self,
        source: &Dom,
        source_node: NodeId,
        parent: Option<NodeId>,
    ) -> Result<NodeId> {
        let node_type = match &source.nodes[source_node.0].node_type {
            NodeType::Document => {
                return Err(Error::ScriptRuntime(
                    "cannot clone document node into innerHTML target".into(),
                ));
            }
            NodeType::Element(element) => NodeType::Element(element.clone()),
            NodeType::Text(text) => NodeType::Text(text.clone()),
        };

        let node = self.create_node(parent, node_type);
        for child in &source.nodes[source_node.0].children {
            let _ = self.clone_subtree_from_dom(source, *child, Some(node))?;
        }
        Ok(node)
    }

    fn value(&self, node_id: NodeId) -> Result<String> {
        let element = self
            .element(node_id)
            .ok_or_else(|| Error::ScriptRuntime("value target is not an element".into()))?;
        Ok(element.value.clone())
    }

    fn set_value(&mut self, node_id: NodeId, value: &str) -> Result<()> {
        if self
            .tag_name(node_id)
            .map(|tag| tag.eq_ignore_ascii_case("select"))
            .unwrap_or(false)
        {
            return self.set_select_value(node_id, value);
        }

        let element = self
            .element_mut(node_id)
            .ok_or_else(|| Error::ScriptRuntime("value target is not an element".into()))?;
        element.value = value.to_string();
        Ok(())
    }

    fn initialize_form_control_values(&mut self) -> Result<()> {
        let nodes = self.all_element_nodes();
        for node in nodes {
            let is_textarea = self
                .tag_name(node)
                .map(|tag| tag.eq_ignore_ascii_case("textarea"))
                .unwrap_or(false);
            if is_textarea {
                let text = self.text_content(node);
                let element = self.element_mut(node).ok_or_else(|| {
                    Error::ScriptRuntime("textarea target is not an element".into())
                })?;
                element.value = text;
                continue;
            }

            let is_select = self
                .tag_name(node)
                .map(|tag| tag.eq_ignore_ascii_case("select"))
                .unwrap_or(false);
            if is_select {
                self.sync_select_value(node)?;
            }
        }
        Ok(())
    }

    fn sync_select_value_for_option(&mut self, option_node: NodeId) -> Result<()> {
        if !self
            .tag_name(option_node)
            .map(|tag| tag.eq_ignore_ascii_case("option"))
            .unwrap_or(false)
        {
            return Ok(());
        }

        let Some(select_node) = self.find_ancestor_by_tag(option_node, "select") else {
            return Ok(());
        };
        self.sync_select_value(select_node)
    }

    fn set_select_value(&mut self, select_node: NodeId, requested: &str) -> Result<()> {
        let tag = self
            .tag_name(select_node)
            .ok_or_else(|| Error::ScriptRuntime("select target is not an element".into()))?;
        if !tag.eq_ignore_ascii_case("select") {
            return Err(Error::ScriptRuntime(
                "set value target is not a select".into(),
            ));
        }

        let mut options = Vec::new();
        self.collect_select_options(select_node, &mut options);

        let mut option_values = Vec::with_capacity(options.len());
        for option in options {
            option_values.push((option, self.option_effective_value(option)?));
        }

        let matched = option_values
            .iter()
            .find(|(_, value)| value == requested)
            .map(|(node, value)| (*node, value.clone()));

        for (option, _) in &option_values {
            let option_element = self
                .element_mut(*option)
                .ok_or_else(|| Error::ScriptRuntime("option target is not an element".into()))?;
            if Some(*option) == matched.as_ref().map(|(node, _)| *node) {
                option_element
                    .attrs
                    .insert("selected".to_string(), "true".to_string());
            } else {
                option_element.attrs.remove("selected");
            }
        }

        let element = self
            .element_mut(select_node)
            .ok_or_else(|| Error::ScriptRuntime("select target is not an element".into()))?;
        element.value = matched.map(|(_, value)| value).unwrap_or_default();
        Ok(())
    }

    fn sync_select_value(&mut self, select_node: NodeId) -> Result<()> {
        let value = self.select_value_from_options(select_node)?;
        let element = self
            .element_mut(select_node)
            .ok_or_else(|| Error::ScriptRuntime("select target is not an element".into()))?;
        element.value = value;
        Ok(())
    }

    fn select_value_from_options(&self, select_node: NodeId) -> Result<String> {
        let tag = self
            .tag_name(select_node)
            .ok_or_else(|| Error::ScriptRuntime("select target is not an element".into()))?;
        if !tag.eq_ignore_ascii_case("select") {
            return Err(Error::ScriptRuntime(
                "select value target is not a select".into(),
            ));
        }

        let mut options = Vec::new();
        self.collect_select_options(select_node, &mut options);
        if options.is_empty() {
            return Ok(String::new());
        }

        let selected = options
            .iter()
            .copied()
            .find(|option| self.attr(*option, "selected").is_some())
            .unwrap_or(options[0]);
        self.option_effective_value(selected)
    }

    fn collect_select_options(&self, node: NodeId, out: &mut Vec<NodeId>) {
        for child in &self.nodes[node.0].children {
            if self
                .tag_name(*child)
                .map(|tag| tag.eq_ignore_ascii_case("option"))
                .unwrap_or(false)
            {
                out.push(*child);
            }
            self.collect_select_options(*child, out);
        }
    }

    fn option_effective_value(&self, option_node: NodeId) -> Result<String> {
        let element = self
            .element(option_node)
            .ok_or_else(|| Error::ScriptRuntime("option target is not an element".into()))?;
        if !element.tag_name.eq_ignore_ascii_case("option") {
            return Err(Error::ScriptRuntime(
                "option target is not an option".into(),
            ));
        }
        if let Some(value) = element.attrs.get("value") {
            return Ok(value.clone());
        }
        Ok(self.text_content(option_node))
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

    fn readonly(&self, node_id: NodeId) -> bool {
        self.element(node_id).map(|e| e.readonly).unwrap_or(false)
    }

    fn required(&self, node_id: NodeId) -> bool {
        self.element(node_id).map(|e| e.required).unwrap_or(false)
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
        let connected = self.is_connected(node_id);
        let (is_option, lowered) = {
            let element = self.element_mut(node_id).ok_or_else(|| {
                Error::ScriptRuntime("setAttribute target is not an element".into())
            })?;
            let is_option = element.tag_name.eq_ignore_ascii_case("option");
            let lowered = name.to_ascii_lowercase();
            element.attrs.insert(lowered.clone(), value.to_string());

            if lowered == "value" {
                element.value = value.to_string();
            } else if lowered == "checked" {
                element.checked = true;
            } else if lowered == "disabled" {
                element.disabled = true;
            } else if lowered == "readonly" {
                element.readonly = true;
            } else if lowered == "required" {
                element.required = true;
            }
            (is_option, lowered)
        };

        if lowered == "id" && connected {
            if let Some(old) = old_id {
                self.unindex_id(&old, node_id);
            }
            if !value.is_empty() {
                self.index_id(value, node_id);
            }
        }

        if is_option && (lowered == "selected" || lowered == "value") {
            self.sync_select_value_for_option(node_id)?;
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
        let connected = self.is_connected(node_id);
        let is_option = {
            let element = self.element_mut(node_id).ok_or_else(|| {
                Error::ScriptRuntime("removeAttribute target is not an element".into())
            })?;
            let is_option = element.tag_name.eq_ignore_ascii_case("option");
            element.attrs.remove(&lowered);

            if lowered == "value" {
                element.value.clear();
            } else if lowered == "checked" {
                element.checked = false;
            } else if lowered == "disabled" {
                element.disabled = false;
            } else if lowered == "readonly" {
                element.readonly = false;
            } else if lowered == "required" {
                element.required = false;
            }
            is_option
        };

        if lowered == "id" && connected {
            if let Some(old) = old_id {
                self.unindex_id(&old, node_id);
            }
        }

        if is_option && (lowered == "selected" || lowered == "value") {
            self.sync_select_value_for_option(node_id)?;
        }

        Ok(())
    }

    fn append_child(&mut self, parent: NodeId, child: NodeId) -> Result<()> {
        if !self.can_have_children(parent) {
            return Err(Error::ScriptRuntime(
                "appendChild target cannot have children".into(),
            ));
        }
        if child == self.root || child == parent {
            return Err(Error::ScriptRuntime("invalid appendChild node".into()));
        }
        if !self.is_valid_node(child) {
            return Err(Error::ScriptRuntime("appendChild node is invalid".into()));
        }

        // Prevent cycles: parent must not be inside child's subtree.
        let mut cursor = Some(parent);
        while let Some(node) = cursor {
            if node == child {
                return Err(Error::ScriptRuntime(
                    "appendChild would create a cycle".into(),
                ));
            }
            cursor = self.parent(node);
        }

        if let Some(old_parent) = self.parent(child) {
            self.nodes[old_parent.0].children.retain(|id| *id != child);
        }
        self.nodes[child.0].parent = Some(parent);
        self.nodes[parent.0].children.push(child);
        self.rebuild_id_index();
        Ok(())
    }

    fn prepend_child(&mut self, parent: NodeId, child: NodeId) -> Result<()> {
        let reference = self.nodes[parent.0].children.first().copied();
        if let Some(reference) = reference {
            self.insert_before(parent, child, reference)
        } else {
            self.append_child(parent, child)
        }
    }

    fn insert_before(&mut self, parent: NodeId, child: NodeId, reference: NodeId) -> Result<()> {
        if !self.can_have_children(parent) {
            return Err(Error::ScriptRuntime(
                "insertBefore target cannot have children".into(),
            ));
        }
        if child == self.root || child == parent {
            return Err(Error::ScriptRuntime("invalid insertBefore node".into()));
        }
        if !self.is_valid_node(child) || !self.is_valid_node(reference) {
            return Err(Error::ScriptRuntime("insertBefore node is invalid".into()));
        }
        if self.parent(reference) != Some(parent) {
            return Err(Error::ScriptRuntime(
                "insertBefore reference is not a direct child".into(),
            ));
        }
        if child == reference {
            return Ok(());
        }

        // Prevent cycles: parent must not be inside child's subtree.
        let mut cursor = Some(parent);
        while let Some(node) = cursor {
            if node == child {
                return Err(Error::ScriptRuntime(
                    "insertBefore would create a cycle".into(),
                ));
            }
            cursor = self.parent(node);
        }

        if let Some(old_parent) = self.parent(child) {
            self.nodes[old_parent.0].children.retain(|id| *id != child);
        }

        let Some(index) = self.nodes[parent.0]
            .children
            .iter()
            .position(|id| *id == reference)
        else {
            return Err(Error::ScriptRuntime(
                "insertBefore reference is missing".into(),
            ));
        };

        self.nodes[child.0].parent = Some(parent);
        self.nodes[parent.0].children.insert(index, child);
        self.rebuild_id_index();
        Ok(())
    }

    fn insert_after(&mut self, target: NodeId, child: NodeId) -> Result<()> {
        let Some(parent) = self.parent(target) else {
            return Ok(());
        };
        let pos = self.nodes[parent.0]
            .children
            .iter()
            .position(|id| *id == target)
            .ok_or_else(|| Error::ScriptRuntime("after target is detached".into()))?;
        let next = self.nodes[parent.0].children.get(pos + 1).copied();
        if let Some(next) = next {
            self.insert_before(parent, child, next)
        } else {
            self.append_child(parent, child)
        }
    }

    fn replace_with(&mut self, target: NodeId, child: NodeId) -> Result<()> {
        let Some(parent) = self.parent(target) else {
            return Ok(());
        };
        if target == child {
            return Ok(());
        }
        self.insert_before(parent, child, target)?;
        self.remove_child(parent, target)
    }

    fn insert_adjacent_node(
        &mut self,
        target: NodeId,
        position: InsertAdjacentPosition,
        node: NodeId,
    ) -> Result<()> {
        match position {
            InsertAdjacentPosition::BeforeBegin => {
                if let Some(parent) = self.parent(target) {
                    self.insert_before(parent, node, target)?;
                }
                Ok(())
            }
            InsertAdjacentPosition::AfterBegin => self.prepend_child(target, node),
            InsertAdjacentPosition::BeforeEnd => self.append_child(target, node),
            InsertAdjacentPosition::AfterEnd => self.insert_after(target, node),
        }
    }

    fn remove_child(&mut self, parent: NodeId, child: NodeId) -> Result<()> {
        if self.parent(child) != Some(parent) {
            return Err(Error::ScriptRuntime(
                "removeChild target is not a direct child".into(),
            ));
        }
        self.nodes[parent.0].children.retain(|id| *id != child);
        self.nodes[child.0].parent = None;
        self.rebuild_id_index();
        Ok(())
    }

    fn remove_node(&mut self, node: NodeId) -> Result<()> {
        if node == self.root {
            return Err(Error::ScriptRuntime("cannot remove document root".into()));
        }
        let Some(parent) = self.parent(node) else {
            return Ok(());
        };
        self.remove_child(parent, node)
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

    fn offset_left(&self, node_id: NodeId) -> Result<i64> {
        if self.element(node_id).is_none() {
            return Err(Error::ScriptRuntime(
                "offsetLeft target is not an element".into(),
            ));
        }
        Ok(0)
    }

    fn offset_top(&self, node_id: NodeId) -> Result<i64> {
        if self.element(node_id).is_none() {
            return Err(Error::ScriptRuntime(
                "offsetTop target is not an element".into(),
            ));
        }
        Ok(0)
    }

    fn offset_width(&self, node_id: NodeId) -> Result<i64> {
        if self.element(node_id).is_none() {
            return Err(Error::ScriptRuntime(
                "offsetWidth target is not an element".into(),
            ));
        }
        Ok(0)
    }

    fn offset_height(&self, node_id: NodeId) -> Result<i64> {
        if self.element(node_id).is_none() {
            return Err(Error::ScriptRuntime(
                "offsetHeight target is not an element".into(),
            ));
        }
        Ok(0)
    }

    fn scroll_width(&self, node_id: NodeId) -> Result<i64> {
        if self.element(node_id).is_none() {
            return Err(Error::ScriptRuntime(
                "scrollWidth target is not an element".into(),
            ));
        }
        Ok(0)
    }

    fn scroll_height(&self, node_id: NodeId) -> Result<i64> {
        if self.element(node_id).is_none() {
            return Err(Error::ScriptRuntime(
                "scrollHeight target is not an element".into(),
            ));
        }
        Ok(0)
    }

    fn scroll_left(&self, node_id: NodeId) -> Result<i64> {
        if self.element(node_id).is_none() {
            return Err(Error::ScriptRuntime(
                "scrollLeft target is not an element".into(),
            ));
        }
        Ok(0)
    }

    fn scroll_top(&self, node_id: NodeId) -> Result<i64> {
        if self.element(node_id).is_none() {
            return Err(Error::ScriptRuntime(
                "scrollTop target is not an element".into(),
            ));
        }
        Ok(0)
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
        let groups = parse_selector_groups(selector)?;

        if groups.len() == 1 && groups[0].len() == 1 {
            if let Some(id) = groups[0][0].step.id_only() {
                return Ok(self.by_id_all(id));
            }
        }

        let mut ids = Vec::new();
        self.collect_elements_dfs(self.root, &mut ids);

        let mut seen = HashSet::new();
        let mut matched = Vec::new();
        for candidate in ids {
            if groups
                .iter()
                .any(|steps| self.matches_selector_chain(candidate, steps))
                && seen.insert(candidate)
            {
                matched.push(candidate);
            }
        }
        Ok(matched)
    }

    fn query_selector_from(&self, root: &NodeId, selector: &str) -> Result<Option<NodeId>> {
        let all = self.query_selector_all_from(root, selector)?;
        Ok(all.into_iter().next())
    }

    fn query_selector_all_from(&self, root: &NodeId, selector: &str) -> Result<Vec<NodeId>> {
        let groups = parse_selector_groups(selector)?;

        let mut ids = Vec::new();
        self.collect_elements_descendants_dfs(*root, &mut ids);

        let mut seen = HashSet::new();
        let mut matched = Vec::new();
        for candidate in ids {
            if groups
                .iter()
                .any(|steps| self.matches_selector_chain(candidate, steps))
                && seen.insert(candidate)
            {
                matched.push(candidate);
            }
        }
        Ok(matched)
    }

    fn matches_selector(&self, node_id: NodeId, selector: &str) -> Result<bool> {
        if self.element(node_id).is_none() {
            return Ok(false);
        }

        let groups = parse_selector_groups(selector)?;
        Ok(groups
            .iter()
            .any(|steps| self.matches_selector_chain(node_id, steps)))
    }

    fn closest(&self, node_id: NodeId, selector: &str) -> Result<Option<NodeId>> {
        if self.element(node_id).is_none() {
            return Ok(None);
        }

        let groups = parse_selector_groups(selector)?;
        let mut cursor = Some(node_id);
        while let Some(current) = cursor {
            if groups
                .iter()
                .any(|steps| self.matches_selector_chain(current, steps))
            {
                return Ok(Some(current));
            }
            cursor = self.parent(current);
        }
        Ok(None)
    }

    fn can_have_children(&self, node_id: NodeId) -> bool {
        matches!(
            self.nodes.get(node_id.0).map(|n| &n.node_type),
            Some(NodeType::Document | NodeType::Element(_))
        )
    }

    fn is_valid_node(&self, node_id: NodeId) -> bool {
        node_id.0 < self.nodes.len()
    }

    fn is_connected(&self, node_id: NodeId) -> bool {
        let mut cursor = Some(node_id);
        while let Some(node) = cursor {
            if node == self.root {
                return true;
            }
            cursor = self.parent(node);
        }
        false
    }

    fn rebuild_id_index(&mut self) {
        let mut next = HashMap::new();
        let mut stack = vec![self.root];
        while let Some(node) = stack.pop() {
            match &self.nodes[node.0].node_type {
                NodeType::Element(element) => {
                    if let Some(id) = element.attrs.get("id") {
                        Self::index_id_map(&mut next, id, node);
                    }
                }
                NodeType::Document | NodeType::Text(_) => {}
            }
            for child in self.nodes[node.0].children.iter().rev() {
                stack.push(*child);
            }
        }
        self.id_index = next;
    }

    fn index_id_map(next: &mut HashMap<String, Vec<NodeId>>, id: &str, node_id: NodeId) {
        if id.is_empty() {
            return;
        }
        next.entry(id.to_string()).or_default().push(node_id);
    }

    fn collect_elements_dfs(&self, node_id: NodeId, out: &mut Vec<NodeId>) {
        if matches!(self.nodes[node_id.0].node_type, NodeType::Element(_)) {
            out.push(node_id);
        }
        for child in &self.nodes[node_id.0].children {
            self.collect_elements_dfs(*child, out);
        }
    }

    fn collect_elements_descendants_dfs(&self, node_id: NodeId, out: &mut Vec<NodeId>) {
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
                SelectorCombinator::AdjacentSibling => self
                    .previous_element_sibling(current)
                    .filter(|sibling| self.matches_step(*sibling, prev_step)),
                SelectorCombinator::GeneralSibling => {
                    let mut cursor = self.previous_element_sibling(current);
                    let mut found = None;
                    while let Some(sibling) = cursor {
                        if self.matches_step(sibling, prev_step) {
                            found = Some(sibling);
                            break;
                        }
                        cursor = self.previous_element_sibling(sibling);
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

        if !step.universal {
            if let Some(tag) = &step.tag {
                if !element.tag_name.eq_ignore_ascii_case(tag) {
                    return false;
                }
            }
        } else if step.tag.is_some() {
            return false;
        }

        if let Some(id) = &step.id {
            if element.attrs.get("id") != Some(id) {
                return false;
            }
        }

        if step
            .classes
            .iter()
            .any(|class_name| !has_class(element, class_name))
        {
            return false;
        }

        for cond in &step.attrs {
            let matched = match cond {
                SelectorAttrCondition::Exists { key } => element.attrs.contains_key(key),
                SelectorAttrCondition::Eq { key, value } => element.attrs.get(key) == Some(value),
                SelectorAttrCondition::StartsWith { key, value } => element
                    .attrs
                    .get(key)
                    .is_some_and(|attr| attr.starts_with(value)),
                SelectorAttrCondition::EndsWith { key, value } => element
                    .attrs
                    .get(key)
                    .is_some_and(|attr| attr.ends_with(value)),
                SelectorAttrCondition::Contains { key, value } => element
                    .attrs
                    .get(key)
                    .is_some_and(|attr| attr.contains(value)),
                SelectorAttrCondition::Includes { key, value } => element
                    .attrs
                    .get(key)
                    .is_some_and(|attr| attr.split_whitespace().any(|token| token == value)),
                SelectorAttrCondition::DashMatch { key, value } => element
                    .attrs
                    .get(key)
                    .is_some_and(|attr| attr == value || attr.starts_with(&format!("{value}-"))),
            };
            if !matched {
                return false;
            }
        }

        for pseudo in &step.pseudo_classes {
            let matched = match pseudo {
                SelectorPseudoClass::FirstChild => self.is_first_element_child(node_id),
                SelectorPseudoClass::LastChild => self.is_last_element_child(node_id),
                SelectorPseudoClass::FirstOfType => self.is_first_of_type(node_id),
                SelectorPseudoClass::LastOfType => self.is_last_of_type(node_id),
                SelectorPseudoClass::OnlyChild => self.is_only_element_child(node_id),
                SelectorPseudoClass::OnlyOfType => self.is_only_of_type(node_id),
                SelectorPseudoClass::Checked => {
                    self.element(node_id).is_some_and(|node| node.checked)
                }
                SelectorPseudoClass::Disabled => {
                    self.element(node_id).is_some_and(|node| node.disabled)
                }
                SelectorPseudoClass::Enabled => {
                    self.element(node_id).is_some_and(|node| !node.disabled)
                }
                SelectorPseudoClass::Required => {
                    self.element(node_id).is_some_and(|node| node.required)
                }
                SelectorPseudoClass::Optional => {
                    self.element(node_id).is_none_or(|node| !node.required)
                }
                SelectorPseudoClass::Readonly => {
                    self.element(node_id).is_some_and(|node| node.readonly)
                }
                SelectorPseudoClass::Readwrite => {
                    self.element(node_id).is_none_or(|node| !node.readonly)
                }
                SelectorPseudoClass::Empty => self.nodes[node_id.0].children.is_empty(),
                SelectorPseudoClass::Focus => self
                    .element(node_id)
                    .is_some_and(|_| self.active_element == Some(node_id)),
                SelectorPseudoClass::FocusWithin => {
                    if self.active_element == Some(node_id) {
                        true
                    } else {
                        self.active_element
                            .is_some_and(|active| self.is_descendant_of(active, node_id))
                    }
                }
                SelectorPseudoClass::Active => self
                    .element(node_id)
                    .is_some_and(|_| self.active_pseudo_element == Some(node_id)),
                SelectorPseudoClass::NthOfType(selector) => {
                    self.is_nth_element_of_type(node_id, selector)
                }
                SelectorPseudoClass::NthLastOfType(selector) => {
                    self.is_nth_last_element_of_type(node_id, selector)
                }
                SelectorPseudoClass::Is(inners) | SelectorPseudoClass::Where(inners) => {
                    inners
                        .iter()
                        .any(|inner| self.matches_selector_chain(node_id, inner))
                }
                SelectorPseudoClass::Has(inners) => {
                    let mut descendants = Vec::new();
                    self.collect_elements_descendants_dfs(node_id, &mut descendants);
                    inners.iter().any(|inner| {
                        descendants
                            .iter()
                            .any(|target| self.matches_selector_chain(*target, inner))
                    })
                }
                SelectorPseudoClass::NthLastChild(selector) => {
                    self.is_nth_last_element_child(node_id, selector)
                }
                SelectorPseudoClass::NthChild(selector) => {
                    self.is_nth_element_child(node_id, selector)
                }
                SelectorPseudoClass::Not(inners) => !inners
                    .iter()
                    .any(|inner| self.matches_selector_chain(node_id, inner)),
            };
            if !matched {
                return false;
            }
        }

        true
    }

    fn is_first_element_child(&self, node_id: NodeId) -> bool {
        self.previous_element_sibling(node_id).is_none()
    }

    fn is_last_element_child(&self, node_id: NodeId) -> bool {
        self.next_element_sibling(node_id).is_none()
    }

    fn is_only_element_child(&self, node_id: NodeId) -> bool {
        let Some(parent) = self.parent(node_id) else {
            return false;
        };
        let mut count = 0usize;
        for child in &self.nodes[parent.0].children {
            if self.element(*child).is_some() {
                count += 1;
            }
        }
        count == 1
    }

    fn is_nth_element_child(&self, node_id: NodeId, selector: &NthChildSelector) -> bool {
        let Some(index) = self.element_index(node_id) else {
            return false;
        };
        self.is_nth_index_element_child(index, selector)
    }

    fn is_nth_last_element_child(&self, node_id: NodeId, selector: &NthChildSelector) -> bool {
        let Some(parent) = self.parent(node_id) else {
            return false;
        };
        let mut index = 0usize;
        let mut target = None;
        for child in &self.nodes[parent.0].children {
            if self.element(*child).is_none() {
                continue;
            }
            index += 1;
            if *child == node_id {
                target = Some(index);
            }
        }
        let Some(target) = target else {
            return false;
        };
        let total = index;
        let index_from_last = (total + 1) - target;
        self.is_nth_index_element_child(index_from_last, selector)
    }

    fn is_nth_element_of_type(&self, node_id: NodeId, selector: &NthChildSelector) -> bool {
        let Some(parent) = self.parent(node_id) else {
            return false;
        };
        let Some(tag_name) = self.tag_name(node_id) else {
            return false;
        };
        let mut index = 0usize;
        let mut target = None;
        for child in &self.nodes[parent.0].children {
            let Some(element) = self.element(*child) else {
                continue;
            };
            if element.tag_name != tag_name {
                continue;
            }
            index += 1;
            if *child == node_id {
                target = Some(index);
            }
        }
        let Some(target) = target else {
            return false;
        };
        self.is_nth_index_element_child(target, selector)
    }

    fn is_nth_last_element_of_type(&self, node_id: NodeId, selector: &NthChildSelector) -> bool {
        let Some(parent) = self.parent(node_id) else {
            return false;
        };
        let Some(tag_name) = self.tag_name(node_id) else {
            return false;
        };
        let mut index = 0usize;
        let mut target = None;
        for child in &self.nodes[parent.0].children {
            let Some(element) = self.element(*child) else {
                continue;
            };
            if element.tag_name != tag_name {
                continue;
            }
            index += 1;
            if *child == node_id {
                target = Some(index);
            }
        }
        let Some(target) = target else {
            return false;
        };
        let total = index;
        let index_from_last = (total + 1) - target;
        self.is_nth_index_element_child(index_from_last, selector)
    }

    fn is_first_of_type(&self, node_id: NodeId) -> bool {
        let Some(parent) = self.parent(node_id) else {
            return false;
        };
        let Some(tag_name) = self.tag_name(node_id) else {
            return false;
        };

        for child in &self.nodes[parent.0].children {
            let Some(element) = self.element(*child) else {
                continue;
            };
            if element.tag_name == tag_name {
                return *child == node_id;
            }
        }
        false
    }

    fn is_only_of_type(&self, node_id: NodeId) -> bool {
        let Some(parent) = self.parent(node_id) else {
            return false;
        };
        let Some(tag_name) = self.tag_name(node_id) else {
            return false;
        };
        let mut same_type_count = 0usize;
        for child in &self.nodes[parent.0].children {
            let Some(element) = self.element(*child) else {
                continue;
            };
            if element.tag_name == tag_name {
                same_type_count += 1;
            }
        }
        same_type_count == 1
    }

    fn is_last_of_type(&self, node_id: NodeId) -> bool {
        let Some(parent) = self.parent(node_id) else {
            return false;
        };
        let Some(tag_name) = self.tag_name(node_id) else {
            return false;
        };

        for child in self.nodes[parent.0].children.iter().rev() {
            let Some(element) = self.element(*child) else {
                continue;
            };
            if element.tag_name == tag_name {
                return *child == node_id;
            }
        }
        false
    }

    fn is_nth_index_element_child(&self, index: usize, selector: &NthChildSelector) -> bool {
        match selector {
            NthChildSelector::Exact(expected) => index == *expected,
            NthChildSelector::Odd => index % 2 == 1,
            NthChildSelector::Even => index % 2 == 0,
            NthChildSelector::AnPlusB(a, b) => {
                let index = index as i64;
                let diff = index - *b;
                if *a == 0 {
                    return diff == 0;
                }
                diff % *a == 0 && (diff / *a) >= 0
            }
        }
    }

    fn element_index(&self, node_id: NodeId) -> Option<usize> {
        let parent = self.parent(node_id)?;
        let mut index = 0usize;
        for child in &self.nodes[parent.0].children {
            if self.element(*child).is_none() {
                continue;
            }
            index += 1;
            if *child == node_id {
                return Some(index);
            }
        }
        None
    }

    fn next_element_sibling(&self, node_id: NodeId) -> Option<NodeId> {
        let parent = self.parent(node_id)?;
        let children = &self.nodes[parent.0].children;
        let pos = children.iter().position(|id| *id == node_id)?;
        for sibling in children.iter().skip(pos + 1) {
            if self.element(*sibling).is_some() {
                return Some(*sibling);
            }
        }
        None
    }

    fn previous_element_sibling(&self, node_id: NodeId) -> Option<NodeId> {
        let parent = self.parent(node_id)?;
        let children = &self.nodes[parent.0].children;
        let pos = children.iter().position(|id| *id == node_id)?;
        for sibling in children[..pos].iter().rev() {
            if self.element(*sibling).is_some() {
                return Some(*sibling);
            }
        }
        None
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

    let mut start = 0usize;
    let mut i = 0usize;
    let bytes = style_attr.as_bytes();
    let mut paren_depth = 0isize;
    let mut quote: Option<u8> = None;

    while i < bytes.len() {
        let ch = bytes[i];
        match (quote, ch) {
            (Some(q), _) if ch == b'\\' => {
                if i + 1 < bytes.len() {
                    i += 2;
                    continue;
                }
            }
            (Some(q), _) if ch == q => {
                quote = None;
            }
            (Some(_), _) => {}
            (None, b'\'') | (None, b'"') => {
                quote = Some(ch);
            }
            (None, b'(') => paren_depth += 1,
            (None, b')') => paren_depth = paren_depth.saturating_sub(1),
            (None, b';') if paren_depth == 0 => {
                let decl = &style_attr[start..i];
                push_style_declaration(decl, &mut out);
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }

    let decl = &style_attr[start..];
    push_style_declaration(decl, &mut out);

    out
}

fn push_style_declaration(raw_decl: &str, out: &mut Vec<(String, String)>) {
    let decl = raw_decl.trim();
    if decl.is_empty() {
        return;
    }

    let bytes = decl.as_bytes();
    let mut colon = None;
    let mut paren_depth = 0isize;
    let mut quote: Option<u8> = None;
    let mut i = 0usize;

    while i < bytes.len() {
        let ch = bytes[i];
        match (quote, ch) {
            (Some(q), _) if ch == b'\\' => {
                if i + 1 < bytes.len() {
                    i += 2;
                    continue;
                }
            }
            (Some(q), _) if ch == q => quote = None,
            (Some(_), _) => {}
            (None, b'\'') | (None, b'"') => quote = Some(ch),
            (None, b'(') => paren_depth += 1,
            (None, b')') => paren_depth = paren_depth.saturating_sub(1),
            (None, b':') if paren_depth == 0 && colon.is_none() => {
                colon = Some(i);
                break;
            }
            _ => {}
        }
        i += 1;
    }

    let Some(colon) = colon else {
        return;
    };

    let name = decl[..colon].trim().to_ascii_lowercase();
    if name.is_empty() {
        return;
    }

    let value = decl[colon + 1..].trim().to_string();

    if let Some(pos) = out.iter().position(|(existing, _)| existing == &name) {
        out[pos].1 = value;
    } else {
        out.push((name, value));
    }
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

fn format_float(value: f64) -> String {
    let mut out = format!("{:.16}", value);
    while out.contains('.') && out.ends_with('0') {
        out.pop();
    }
    if out.ends_with('.') {
        out.pop();
    }
    out
}

fn parse_js_parse_float(src: &str) -> f64 {
    let src = src.trim_start();
    if src.is_empty() {
        return f64::NAN;
    }

    let bytes = src.as_bytes();
    let mut i = 0usize;

    if matches!(bytes.get(i), Some(b'+') | Some(b'-')) {
        i += 1;
    }

    if src[i..].starts_with("Infinity") {
        return if matches!(bytes.first(), Some(b'-')) {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        };
    }

    let mut int_digits = 0usize;
    while matches!(bytes.get(i), Some(b) if b.is_ascii_digit()) {
        int_digits += 1;
        i += 1;
    }

    let mut frac_digits = 0usize;
    if bytes.get(i) == Some(&b'.') {
        i += 1;
        while matches!(bytes.get(i), Some(b) if b.is_ascii_digit()) {
            frac_digits += 1;
            i += 1;
        }
    }

    if int_digits + frac_digits == 0 {
        return f64::NAN;
    }

    if matches!(bytes.get(i), Some(b'e') | Some(b'E')) {
        let exp_start = i;
        i += 1;
        if matches!(bytes.get(i), Some(b'+') | Some(b'-')) {
            i += 1;
        }

        let mut exp_digits = 0usize;
        while matches!(bytes.get(i), Some(b) if b.is_ascii_digit()) {
            exp_digits += 1;
            i += 1;
        }

        if exp_digits == 0 {
            i = exp_start;
        }
    }

    src[..i].parse::<f64>().unwrap_or(f64::NAN)
}

fn parse_js_parse_int(src: &str, radix: Option<i64>) -> f64 {
    let src = src.trim_start();
    if src.is_empty() {
        return f64::NAN;
    }

    let bytes = src.as_bytes();
    let mut i = 0usize;
    let negative = if matches!(bytes.get(i), Some(b'+') | Some(b'-')) {
        let is_negative = bytes[i] == b'-';
        i += 1;
        is_negative
    } else {
        false
    };

    let mut radix = radix.unwrap_or(0);
    if radix != 0 {
        if !(2..=36).contains(&radix) {
            return f64::NAN;
        }
    } else {
        radix = 10;
        if src[i..].starts_with("0x") || src[i..].starts_with("0X") {
            radix = 16;
            i += 2;
        }
    }

    if radix == 16 && (src[i..].starts_with("0x") || src[i..].starts_with("0X")) {
        i += 2;
    }

    let mut parsed_any = false;
    let mut value = 0.0f64;
    for ch in src[i..].chars() {
        let Some(digit) = ch.to_digit(36) else {
            break;
        };
        if i64::from(digit) >= radix {
            break;
        }
        parsed_any = true;
        value = (value * (radix as f64)) + (digit as f64);
    }

    if !parsed_any {
        return f64::NAN;
    }

    if negative { -value } else { value }
}

fn encode_binary_string_to_base64(src: &str) -> Result<String> {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut bytes = Vec::with_capacity(src.len());
    for ch in src.chars() {
        let code = ch as u32;
        if code > 0xFF {
            return Err(Error::ScriptRuntime(
                "btoa input contains non-Latin1 character".into(),
            ));
        }
        bytes.push(code as u8);
    }

    let mut out = String::new();
    let mut i = 0usize;
    while i + 3 <= bytes.len() {
        let b0 = bytes[i];
        let b1 = bytes[i + 1];
        let b2 = bytes[i + 2];

        out.push(TABLE[(b0 >> 2) as usize] as char);
        out.push(TABLE[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
        out.push(TABLE[(((b1 & 0x0F) << 2) | (b2 >> 6)) as usize] as char);
        out.push(TABLE[(b2 & 0x3F) as usize] as char);
        i += 3;
    }

    let rem = bytes.len().saturating_sub(i);
    if rem == 1 {
        let b0 = bytes[i];
        out.push(TABLE[(b0 >> 2) as usize] as char);
        out.push(TABLE[((b0 & 0x03) << 4) as usize] as char);
        out.push('=');
        out.push('=');
    } else if rem == 2 {
        let b0 = bytes[i];
        let b1 = bytes[i + 1];
        out.push(TABLE[(b0 >> 2) as usize] as char);
        out.push(TABLE[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
        out.push(TABLE[((b1 & 0x0F) << 2) as usize] as char);
        out.push('=');
    }

    Ok(out)
}

fn decode_base64_to_binary_string(src: &str) -> Result<String> {
    let mut bytes: Vec<u8> = src
        .bytes()
        .filter(|b| !b.is_ascii_whitespace())
        .collect();
    if bytes.is_empty() {
        return Ok(String::new());
    }

    match bytes.len() % 4 {
        0 => {}
        2 => bytes.extend_from_slice(b"=="),
        3 => bytes.push(b'='),
        _ => {
            return Err(Error::ScriptRuntime("atob invalid base64 input".into()));
        }
    }

    let mut out = Vec::new();
    let mut i = 0usize;
    while i < bytes.len() {
        let b0 = bytes[i];
        let b1 = bytes[i + 1];
        let b2 = bytes[i + 2];
        let b3 = bytes[i + 3];

        let v0 = decode_base64_char(b0)?;
        let v1 = decode_base64_char(b1)?;
        out.push((v0 << 2) | (v1 >> 4));

        if b2 == b'=' {
            if b3 != b'=' {
                return Err(Error::ScriptRuntime("atob invalid base64 input".into()));
            }
            i += 4;
            continue;
        }

        let v2 = decode_base64_char(b2)?;
        out.push(((v1 & 0x0F) << 4) | (v2 >> 2));

        if b3 == b'=' {
            i += 4;
            continue;
        }

        let v3 = decode_base64_char(b3)?;
        out.push(((v2 & 0x03) << 6) | v3);
        i += 4;
    }

    Ok(out.into_iter().map(char::from).collect())
}

fn decode_base64_char(ch: u8) -> Result<u8> {
    let value = match ch {
        b'A'..=b'Z' => ch - b'A',
        b'a'..=b'z' => ch - b'a' + 26,
        b'0'..=b'9' => ch - b'0' + 52,
        b'+' => 62,
        b'/' => 63,
        _ => {
            return Err(Error::ScriptRuntime("atob invalid base64 input".into()));
        }
    };
    Ok(value)
}

fn encode_uri_like(src: &str, component: bool) -> String {
    let mut out = String::new();
    for b in src.as_bytes() {
        if is_unescaped_uri_byte(*b, component) {
            out.push(*b as char);
        } else {
            out.push('%');
            out.push(to_hex_upper((*b >> 4) & 0x0F));
            out.push(to_hex_upper(*b & 0x0F));
        }
    }
    out
}

fn decode_uri_like(src: &str, component: bool) -> Result<String> {
    let preserve_reserved = !component;
    let bytes = src.as_bytes();
    let mut out = String::new();
    let mut i = 0usize;

    while i < bytes.len() {
        if bytes[i] != b'%' {
            let ch = src[i..].chars().next().ok_or_else(|| {
                Error::ScriptRuntime("malformed URI sequence".into())
            })?;
            out.push(ch);
            i += ch.len_utf8();
            continue;
        }

        let first = parse_percent_byte(bytes, i)?;
        if first < 0x80 {
            let ch = first as char;
            if preserve_reserved && is_decode_uri_reserved_char(ch) {
                let raw = src
                    .get(i..i + 3)
                    .ok_or_else(|| Error::ScriptRuntime("malformed URI sequence".into()))?;
                out.push_str(raw);
            } else {
                out.push(ch);
            }
            i += 3;
            continue;
        }

        let len = utf8_sequence_len(first)
            .ok_or_else(|| Error::ScriptRuntime("malformed URI sequence".into()))?;
        let mut raw_end = i + 3;
        let mut chunk = Vec::with_capacity(len);
        chunk.push(first);
        for _ in 1..len {
            if raw_end >= bytes.len() || bytes[raw_end] != b'%' {
                return Err(Error::ScriptRuntime("malformed URI sequence".into()));
            }
            chunk.push(parse_percent_byte(bytes, raw_end)?);
            raw_end += 3;
        }
        let decoded = std::str::from_utf8(&chunk)
            .map_err(|_| Error::ScriptRuntime("malformed URI sequence".into()))?;
        out.push_str(decoded);
        i = raw_end;
    }

    Ok(out)
}

fn js_escape(src: &str) -> String {
    let mut out = String::new();
    for unit in src.encode_utf16() {
        if unit <= 0x7F && is_unescaped_legacy_escape_byte(unit as u8) {
            out.push(unit as u8 as char);
            continue;
        }

        if unit <= 0xFF {
            let value = unit as u8;
            out.push('%');
            out.push(to_hex_upper((value >> 4) & 0x0F));
            out.push(to_hex_upper(value & 0x0F));
            continue;
        }

        out.push('%');
        out.push('u');
        out.push(to_hex_upper(((unit >> 12) & 0x0F) as u8));
        out.push(to_hex_upper(((unit >> 8) & 0x0F) as u8));
        out.push(to_hex_upper(((unit >> 4) & 0x0F) as u8));
        out.push(to_hex_upper((unit & 0x0F) as u8));
    }
    out
}

fn js_unescape(src: &str) -> String {
    let bytes = src.as_bytes();
    let mut units: Vec<u16> = Vec::with_capacity(src.len());
    let mut i = 0usize;

    while i < bytes.len() {
        if bytes[i] == b'%' {
            if i + 5 < bytes.len()
                && matches!(bytes[i + 1], b'u' | b'U')
                && from_hex_digit(bytes[i + 2]).is_some()
                && from_hex_digit(bytes[i + 3]).is_some()
                && from_hex_digit(bytes[i + 4]).is_some()
                && from_hex_digit(bytes[i + 5]).is_some()
            {
                let u = ((from_hex_digit(bytes[i + 2]).unwrap_or(0) as u16) << 12)
                    | ((from_hex_digit(bytes[i + 3]).unwrap_or(0) as u16) << 8)
                    | ((from_hex_digit(bytes[i + 4]).unwrap_or(0) as u16) << 4)
                    | (from_hex_digit(bytes[i + 5]).unwrap_or(0) as u16);
                units.push(u);
                i += 6;
                continue;
            }

            if i + 2 < bytes.len()
                && from_hex_digit(bytes[i + 1]).is_some()
                && from_hex_digit(bytes[i + 2]).is_some()
            {
                let u = ((from_hex_digit(bytes[i + 1]).unwrap_or(0) << 4)
                    | from_hex_digit(bytes[i + 2]).unwrap_or(0)) as u16;
                units.push(u);
                i += 3;
                continue;
            }
        }

        let ch = src[i..].chars().next().unwrap_or_default();
        let mut buf = [0u16; 2];
        for unit in ch.encode_utf16(&mut buf).iter().copied() {
            units.push(unit);
        }
        i += ch.len_utf8();
    }

    String::from_utf16_lossy(&units)
}

fn is_unescaped_uri_byte(b: u8, component: bool) -> bool {
    if b.is_ascii_alphanumeric() {
        return true;
    }
    if matches!(b, b'-' | b'_' | b'.' | b'!' | b'~' | b'*' | b'\'' | b'(' | b')') {
        return true;
    }
    if !component && matches!(b, b';' | b',' | b'/' | b'?' | b':' | b'@' | b'&' | b'=' | b'+' | b'$' | b'#') {
        return true;
    }
    false
}

fn is_decode_uri_reserved_char(ch: char) -> bool {
    matches!(ch, ';' | ',' | '/' | '?' | ':' | '@' | '&' | '=' | '+' | '$' | '#')
}

fn is_unescaped_legacy_escape_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'*' | b'+' | b'-' | b'.' | b'/' | b'@' | b'_')
}

fn parse_percent_byte(bytes: &[u8], offset: usize) -> Result<u8> {
    if offset + 2 >= bytes.len() || bytes[offset] != b'%' {
        return Err(Error::ScriptRuntime("malformed URI sequence".into()));
    }
    let hi = from_hex_digit(bytes[offset + 1])
        .ok_or_else(|| Error::ScriptRuntime("malformed URI sequence".into()))?;
    let lo = from_hex_digit(bytes[offset + 2])
        .ok_or_else(|| Error::ScriptRuntime("malformed URI sequence".into()))?;
    Ok((hi << 4) | lo)
}

fn utf8_sequence_len(first: u8) -> Option<usize> {
    match first {
        0xC2..=0xDF => Some(2),
        0xE0..=0xEF => Some(3),
        0xF0..=0xF4 => Some(4),
        _ => None,
    }
}

fn from_hex_digit(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

fn to_hex_upper(nibble: u8) -> char {
    match nibble {
        0..=9 => (b'0' + nibble) as char,
        10..=15 => (b'A' + (nibble - 10)) as char,
        _ => '?',
    }
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

#[derive(Debug, Clone, PartialEq, Eq)]
enum SelectorAttrCondition {
    Exists { key: String },
    Eq { key: String, value: String },
    StartsWith { key: String, value: String },
    EndsWith { key: String, value: String },
    Contains { key: String, value: String },
    Includes { key: String, value: String },
    DashMatch { key: String, value: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SelectorPseudoClass {
    FirstChild,
    LastChild,
    FirstOfType,
    LastOfType,
    OnlyChild,
    OnlyOfType,
    Checked,
    Disabled,
    Enabled,
    Required,
    Optional,
    Readonly,
    Readwrite,
    Empty,
    Focus,
    FocusWithin,
    Active,
    NthOfType(NthChildSelector),
    NthLastOfType(NthChildSelector),
    Not(Vec<Vec<SelectorPart>>),
    Is(Vec<Vec<SelectorPart>>),
    Where(Vec<Vec<SelectorPart>>),
    Has(Vec<Vec<SelectorPart>>),
    NthChild(NthChildSelector),
    NthLastChild(NthChildSelector),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum NthChildSelector {
    Exact(usize),
    Odd,
    Even,
    AnPlusB(i64, i64),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct SelectorStep {
    tag: Option<String>,
    universal: bool,
    id: Option<String>,
    classes: Vec<String>,
    attrs: Vec<SelectorAttrCondition>,
    pseudo_classes: Vec<SelectorPseudoClass>,
}

impl SelectorStep {
    fn id_only(&self) -> Option<&str> {
        if !self.universal
            && self.tag.is_none()
            && self.classes.is_empty()
            && self.attrs.is_empty()
            && self.pseudo_classes.is_empty()
        {
            self.id.as_deref()
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SelectorCombinator {
    Descendant,
    Child,
    AdjacentSibling,
    GeneralSibling,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
        if token == ">" || token == "+" || token == "~" {
            if pending_combinator.is_some() || steps.is_empty() {
                return Err(Error::UnsupportedSelector(selector.into()));
            }
            pending_combinator = Some(match token.as_str() {
                ">" => SelectorCombinator::Child,
                "+" => SelectorCombinator::AdjacentSibling,
                "~" => SelectorCombinator::GeneralSibling,
                _ => unreachable!(),
            });
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

fn parse_selector_groups(selector: &str) -> Result<Vec<Vec<SelectorPart>>> {
    let groups = split_selector_groups(selector)?;
    let mut parsed = Vec::with_capacity(groups.len());
    for group in groups {
        parsed.push(parse_selector_chain(&group)?);
    }
    Ok(parsed)
}

fn split_selector_groups(selector: &str) -> Result<Vec<String>> {
    let mut groups = Vec::new();
    let mut current = String::new();
    let mut bracket_depth = 0usize;
    let mut paren_depth = 0usize;

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
            '(' => {
                paren_depth += 1;
                current.push(ch);
            }
            ')' => {
                if paren_depth == 0 {
                    return Err(Error::UnsupportedSelector(selector.into()));
                }
                paren_depth -= 1;
                current.push(ch);
            }
            ',' if bracket_depth == 0 && paren_depth == 0 => {
                let trimmed = current.trim();
                if trimmed.is_empty() {
                    return Err(Error::UnsupportedSelector(selector.into()));
                }
                groups.push(trimmed.to_string());
                current.clear();
            }
            _ => current.push(ch),
        }
    }

    if bracket_depth != 0 {
        return Err(Error::UnsupportedSelector(selector.into()));
    }
    if paren_depth != 0 {
        return Err(Error::UnsupportedSelector(selector.into()));
    }

    let trimmed = current.trim();
    if trimmed.is_empty() {
        return Err(Error::UnsupportedSelector(selector.into()));
    }
    groups.push(trimmed.to_string());
    Ok(groups)
}

fn tokenize_selector(selector: &str) -> Result<Vec<String>> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut bracket_depth = 0usize;
    let mut paren_depth = 0usize;

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
            '(' => {
                paren_depth += 1;
                current.push(ch);
            }
            ')' => {
                if paren_depth == 0 {
                    return Err(Error::UnsupportedSelector(selector.into()));
                }
                paren_depth -= 1;
                current.push(ch);
            }
            '>' | '+' | '~' if bracket_depth == 0 && paren_depth == 0 => {
                if !current.trim().is_empty() {
                    tokens.push(current.trim().to_string());
                }
                current.clear();
                tokens.push(ch.to_string());
            }
            ch if ch.is_ascii_whitespace() && bracket_depth == 0 && paren_depth == 0 => {
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
    if paren_depth != 0 {
        return Err(Error::UnsupportedSelector(selector.into()));
    }

    if !current.trim().is_empty() {
        tokens.push(current.trim().to_string());
    }

    Ok(tokens)
}

fn parse_selector_step(part: &str) -> Result<SelectorStep> {
    let part = part.trim();
    if part.is_empty() {
        return Err(Error::UnsupportedSelector(part.into()));
    }

    let bytes = part.as_bytes();
    let mut i = 0usize;
    let mut step = SelectorStep::default();

    while i < bytes.len() {
        match bytes[i] {
            b'*' => {
                if step.universal {
                    return Err(Error::UnsupportedSelector(part.into()));
                }
                step.universal = true;
                i += 1;
            }
            b'#' => {
                i += 1;
                let Some((id, next)) = parse_selector_ident(part, i) else {
                    return Err(Error::UnsupportedSelector(part.into()));
                };
                if step.id.replace(id).is_some() {
                    return Err(Error::UnsupportedSelector(part.into()));
                }
                i = next;
            }
            b'.' => {
                i += 1;
                let Some((class_name, next)) = parse_selector_ident(part, i) else {
                    return Err(Error::UnsupportedSelector(part.into()));
                };
                step.classes.push(class_name);
                i = next;
            }
            b'[' => {
                let (attr, next) = parse_selector_attr_condition(part, i)?;
                step.attrs.push(attr);
                i = next;
            }
            b':' => {
                let Some((pseudo, next)) = parse_selector_pseudo(part, i) else {
                    return Err(Error::UnsupportedSelector(part.into()));
                };
                step.pseudo_classes.push(pseudo);
                i = next;
            }
            _ => {
                if step.tag.is_some()
                    || step.id.is_some()
                    || !step.classes.is_empty()
                    || step.universal
                {
                    return Err(Error::UnsupportedSelector(part.into()));
                }
                let Some((tag, next)) = parse_selector_ident(part, i) else {
                    return Err(Error::UnsupportedSelector(part.into()));
                };
                step.tag = Some(tag);
                i = next;
            }
        }
    }

    if step.tag.is_none()
        && step.id.is_none()
        && step.classes.is_empty()
        && step.attrs.is_empty()
        && !step.universal
        && step.pseudo_classes.is_empty()
    {
        return Err(Error::UnsupportedSelector(part.into()));
    }
    Ok(step)
}

fn parse_selector_pseudo(part: &str, start: usize) -> Option<(SelectorPseudoClass, usize)> {
    if part.as_bytes().get(start)? != &b':' {
        return None;
    }
    let start = start + 1;
    let tail = part.get(start..)?;
    if let Some(rest) = tail.strip_prefix("first-child") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "first-child".len();
            return Some((SelectorPseudoClass::FirstChild, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("last-child") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "last-child".len();
            return Some((SelectorPseudoClass::LastChild, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("first-of-type") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "first-of-type".len();
            return Some((SelectorPseudoClass::FirstOfType, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("last-of-type") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "last-of-type".len();
            return Some((SelectorPseudoClass::LastOfType, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("only-child") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "only-child".len();
            return Some((SelectorPseudoClass::OnlyChild, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("only-of-type") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "only-of-type".len();
            return Some((SelectorPseudoClass::OnlyOfType, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("checked") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "checked".len();
            return Some((SelectorPseudoClass::Checked, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("disabled") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "disabled".len();
            return Some((SelectorPseudoClass::Disabled, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("required") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "required".len();
            return Some((SelectorPseudoClass::Required, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("optional") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "optional".len();
            return Some((SelectorPseudoClass::Optional, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("read-only") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "read-only".len();
            return Some((SelectorPseudoClass::Readonly, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("readonly") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "readonly".len();
            return Some((SelectorPseudoClass::Readonly, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("read-write") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "read-write".len();
            return Some((SelectorPseudoClass::Readwrite, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("empty") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "empty".len();
            return Some((SelectorPseudoClass::Empty, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("focus-within") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "focus-within".len();
            return Some((SelectorPseudoClass::FocusWithin, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("focus") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "focus".len();
            return Some((SelectorPseudoClass::Focus, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("active") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "active".len();
            return Some((SelectorPseudoClass::Active, consumed));
        }
    }

    if let Some(rest) = tail.strip_prefix("enabled") {
        if rest.is_empty() || is_selector_continuation(rest.as_bytes().first()?) {
            let consumed = start + "enabled".len();
            return Some((SelectorPseudoClass::Enabled, consumed));
        }
    }

    if let Some((inners, next)) = parse_pseudo_selector_list(part, start, "not(") {
        return Some((SelectorPseudoClass::Not(inners), next));
    }

    if let Some((inners, next)) = parse_pseudo_selector_list(part, start, "is(") {
        return Some((SelectorPseudoClass::Is(inners), next));
    }

    if let Some((inners, next)) = parse_pseudo_selector_list(part, start, "where(") {
        return Some((SelectorPseudoClass::Where(inners), next));
    }

    if let Some((inners, next)) = parse_pseudo_selector_list(part, start, "has(") {
        return Some((SelectorPseudoClass::Has(inners), next));
    }

    if let Some(rest) = tail.strip_prefix("nth-last-of-type(") {
        let body = rest;
        let Some(close_pos) = find_matching_paren(body) else {
            return None;
        };
        let raw = body[..close_pos].trim();
        if raw.is_empty() {
            return None;
        }
        let selector = parse_nth_child_selector(raw)?;
        let next = start + "nth-last-of-type(".len() + close_pos + 1;
        if let Some(ch) = part.as_bytes().get(next) {
            if !is_selector_continuation(ch) {
                return None;
            }
        }
        return Some((SelectorPseudoClass::NthLastOfType(selector), next));
    }

    if let Some(rest) = tail.strip_prefix("nth-of-type(") {
        let body = rest;
        let Some(close_pos) = find_matching_paren(body) else {
            return None;
        };
        let raw = body[..close_pos].trim();
        if raw.is_empty() {
            return None;
        }
        let selector = parse_nth_child_selector(raw)?;
        let next = start + "nth-of-type(".len() + close_pos + 1;
        if let Some(ch) = part.as_bytes().get(next) {
            if !is_selector_continuation(ch) {
                return None;
            }
        }
        return Some((SelectorPseudoClass::NthOfType(selector), next));
    }

    if let Some(rest) = tail.strip_prefix("nth-last-child(") {
        let body = rest;
        let Some(close_pos) = find_matching_paren(body) else {
            return None;
        };
        let raw = body[..close_pos].trim();
        if raw.is_empty() {
            return None;
        }
        let selector = parse_nth_child_selector(raw)?;
        let next = start + "nth-last-child(".len() + close_pos + 1;
        if let Some(ch) = part.as_bytes().get(next) {
            if !is_selector_continuation(ch) {
                return None;
            }
        }
        return Some((SelectorPseudoClass::NthLastChild(selector), next));
    }

    if let Some(rest) = tail.strip_prefix("nth-child(") {
        let body = rest;
        let Some(close_pos) = find_matching_paren(body) else {
            return None;
        };
        let raw = body[..close_pos].trim();
        if raw.is_empty() {
            return None;
        }
        let selector = parse_nth_child_selector(raw)?;
        let next = start + "nth-child(".len() + close_pos + 1;
        if let Some(ch) = part.as_bytes().get(next) {
            if !is_selector_continuation(ch) {
                return None;
            }
        }
        return Some((SelectorPseudoClass::NthChild(selector), next));
    }

    None
}

fn parse_pseudo_selector_list(
    part: &str,
    start: usize,
    prefix: &str,
) -> Option<(Vec<Vec<SelectorPart>>, usize)> {
    let Some(rest) = part.get(start..).and_then(|tail| tail.strip_prefix(prefix)) else {
        return None;
    };

    let Some(close_pos) = find_matching_paren(rest) else {
        return None;
    };
    let body = rest[..close_pos].trim();
    if body.is_empty() {
        return None;
    }

    let mut groups = split_selector_groups(body).ok()?;
    if groups.is_empty() {
        return None;
    }

    let mut selectors = Vec::with_capacity(groups.len());
    for group in &mut groups {
        let chain = parse_selector_chain(group.trim()).ok()?;
        if chain.is_empty() {
            return None;
        }
        selectors.push(chain);
    }

    let next = start + prefix.len() + close_pos + 1;
    if let Some(ch) = part.as_bytes().get(next) {
        if !is_selector_continuation(ch) {
            return None;
        }
    }
    Some((selectors, next))
}

fn find_matching_paren(body: &str) -> Option<usize> {
    let mut paren_depth = 1usize;
    let mut bracket_depth = 0usize;
    let mut quote: Option<u8> = None;
    let mut escaped = false;

    for (idx, b) in body.bytes().enumerate() {
        if let Some(q) = quote {
            if escaped {
                escaped = false;
                continue;
            }
            if b == b'\\' {
                escaped = true;
                continue;
            }
            if b == q {
                quote = None;
            }
            continue;
        }

        match b {
            b'\'' | b'"' => quote = Some(b),
            b'[' => {
                bracket_depth += 1;
            }
            b']' => {
                if bracket_depth == 0 {
                    return None;
                }
                bracket_depth -= 1;
            }
            b'(' if bracket_depth == 0 => {
                paren_depth += 1;
            }
            b')' if bracket_depth == 0 => {
                paren_depth = paren_depth.checked_sub(1)?;
                if paren_depth == 0 {
                    return Some(idx);
                }
            }
            _ => {}
        }
    }
    None
}

fn parse_nth_child_selector(raw: &str) -> Option<NthChildSelector> {
    let compact = raw
        .chars()
        .filter(|c| !c.is_ascii_whitespace())
        .collect::<String>()
        .to_ascii_lowercase();
    if compact.is_empty() {
        return None;
    }

    match compact.as_str() {
        "odd" => Some(NthChildSelector::Odd),
        "even" => Some(NthChildSelector::Even),
        other => {
            if other.contains('n') {
                parse_nth_child_expression(other)
            } else {
                if other.starts_with('+') || other.starts_with('-') {
                    None
                } else {
                    let value = other.parse::<usize>().ok()?;
                    if value == 0 {
                        None
                    } else {
                        Some(NthChildSelector::Exact(value))
                    }
                }
            }
        }
    }
}

fn parse_nth_child_expression(raw: &str) -> Option<NthChildSelector> {
    let expr = raw
        .chars()
        .filter(|c| !c.is_ascii_whitespace())
        .collect::<String>();
    let expr = expr.to_ascii_lowercase();
    if expr.matches('n').count() != 1 {
        return None;
    }
    if expr.starts_with(|c: char| c == '+' || c == '-') && expr.len() == 1 {
        return None;
    }

    let n_pos = expr.find('n')?;
    let (a_part, rest) = expr.split_at(n_pos);
    let b_part = &rest[1..];

    let a = match a_part {
        "" => 1,
        "-" => -1,
        "+" => return None,
        _ => a_part.parse::<i64>().ok()?,
    };

    if b_part.is_empty() {
        return Some(NthChildSelector::AnPlusB(a, 0));
    }

    let mut sign = 1;
    let raw_b = if let Some(rest) = b_part.strip_prefix('+') {
        rest
    } else if let Some(rest) = b_part.strip_prefix('-') {
        sign = -1;
        rest
    } else {
        return None;
    };
    if raw_b.is_empty() {
        return None;
    }
    let b = raw_b.parse::<i64>().ok()?;
    Some(NthChildSelector::AnPlusB(a, b * sign))
}

fn is_selector_continuation(next: &u8) -> bool {
    matches!(next, b'.' | b'#' | b'[' | b':')
}

fn parse_selector_ident(src: &str, start: usize) -> Option<(String, usize)> {
    let bytes = src.as_bytes();
    if start >= bytes.len() || !is_selector_ident_char(bytes[start]) {
        return None;
    }
    let mut end = start + 1;
    while end < bytes.len() && is_selector_ident_char(bytes[end]) {
        end += 1;
    }
    Some((src.get(start..end)?.to_string(), end))
}

fn is_selector_ident_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'-'
}

fn parse_selector_attr_condition(
    src: &str,
    open_bracket: usize,
) -> Result<(SelectorAttrCondition, usize)> {
    let bytes = src.as_bytes();
    let mut i = open_bracket + 1;

    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    if i >= bytes.len() {
        return Err(Error::UnsupportedSelector(src.into()));
    }

    let key_start = i;
    while i < bytes.len() {
        if is_selector_attr_name_char(bytes[i]) {
            i += 1;
            continue;
        }
        break;
    }
    if key_start == i {
        return Err(Error::UnsupportedSelector(src.into()));
    }
    let key = src
        .get(key_start..i)
        .ok_or_else(|| Error::UnsupportedSelector(src.into()))?
        .to_ascii_lowercase();

    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    if i >= bytes.len() {
        return Err(Error::UnsupportedSelector(src.into()));
    }

    if bytes[i] == b']' {
        return Ok((SelectorAttrCondition::Exists { key }, i + 1));
    }

    let (op, mut next) = match bytes.get(i) {
        Some(b'=') => (SelectorAttrConditionType::Eq, i + 1),
        Some(b'^') if bytes.get(i + 1) == Some(&b'=') => {
            (SelectorAttrConditionType::StartsWith, i + 2)
        }
        Some(b'$') if bytes.get(i + 1) == Some(&b'=') => {
            (SelectorAttrConditionType::EndsWith, i + 2)
        }
        Some(b'*') if bytes.get(i + 1) == Some(&b'=') => {
            (SelectorAttrConditionType::Contains, i + 2)
        }
        Some(b'~') if bytes.get(i + 1) == Some(&b'=') => {
            (SelectorAttrConditionType::Includes, i + 2)
        }
        Some(b'|') if bytes.get(i + 1) == Some(&b'=') => {
            (SelectorAttrConditionType::DashMatch, i + 2)
        }
        _ => return Err(Error::UnsupportedSelector(src.into())),
    };

    i = next;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    if i >= bytes.len() {
        return Err(Error::UnsupportedSelector(src.into()));
    }

    let (value, after_value) = parse_selector_attr_value(src, i)?;
    next = after_value;

    i = next;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    if i >= bytes.len() || bytes[i] != b']' {
        return Err(Error::UnsupportedSelector(src.into()));
    }

    let cond = match op {
        SelectorAttrConditionType::Eq => SelectorAttrCondition::Eq { key, value },
        SelectorAttrConditionType::StartsWith => SelectorAttrCondition::StartsWith { key, value },
        SelectorAttrConditionType::EndsWith => SelectorAttrCondition::EndsWith { key, value },
        SelectorAttrConditionType::Contains => SelectorAttrCondition::Contains { key, value },
        SelectorAttrConditionType::Includes => SelectorAttrCondition::Includes { key, value },
        SelectorAttrConditionType::DashMatch => SelectorAttrCondition::DashMatch { key, value },
    };

    Ok((cond, i + 1))
}

#[derive(Debug, Clone, Copy)]
enum SelectorAttrConditionType {
    Eq,
    StartsWith,
    EndsWith,
    Contains,
    Includes,
    DashMatch,
}

fn is_selector_attr_name_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'-' || b == b':'
}

fn parse_selector_attr_value(src: &str, start: usize) -> Result<(String, usize)> {
    let bytes = src.as_bytes();
    if start >= bytes.len() {
        return Err(Error::UnsupportedSelector(src.into()));
    }

    if bytes[start] == b'"' || bytes[start] == b'\'' {
        let quote = bytes[start];
        let mut i = start + 1;
        while i < bytes.len() {
            if bytes[i] == b'\\' {
                i = (i + 2).min(bytes.len());
                continue;
            }
            if bytes[i] == quote {
                let raw = src
                    .get(start + 1..i)
                    .ok_or_else(|| Error::UnsupportedSelector(src.into()))?;
                return Ok((unescape_string(raw), i + 1));
            }
            i += 1;
        }
        return Err(Error::UnsupportedSelector(src.into()));
    }

    let start_value = start;
    let mut i = start;
    while i < bytes.len() {
        if bytes[i].is_ascii_whitespace() || bytes[i] == b']' {
            break;
        }
        if bytes[i] == b'\\' {
            i = (i + 2).min(bytes.len());
            continue;
        }
        i += 1;
    }
    if i == start_value {
        return Ok(("".to_string(), i));
    }
    let raw = src
        .get(start_value..i)
        .ok_or_else(|| Error::UnsupportedSelector(src.into()))?;
    Ok((unescape_string(raw), i))
}

#[derive(Debug, Clone, PartialEq)]
enum Value {
    String(String),
    Bool(bool),
    Number(i64),
    Float(f64),
    Array(Rc<RefCell<Vec<Value>>>),
    Date(Rc<RefCell<i64>>),
    Null,
    Undefined,
    Node(NodeId),
    NodeList(Vec<NodeId>),
    FormData(Vec<(String, String)>),
    Function(ScriptHandler),
}

impl Value {
    fn truthy(&self) -> bool {
        match self {
            Self::Bool(v) => *v,
            Self::String(v) => !v.is_empty(),
            Self::Number(v) => *v != 0,
            Self::Float(v) => *v != 0.0,
            Self::Array(_) => true,
            Self::Date(_) => true,
            Self::Null => false,
            Self::Undefined => false,
            Self::Node(_) => true,
            Self::NodeList(nodes) => !nodes.is_empty(),
            Self::FormData(_) => true,
            Self::Function(_) => true,
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
            Self::Float(v) => format_float(*v),
            Self::Array(values) => {
                let values = values.borrow();
                let mut out = String::new();
                for (idx, value) in values.iter().enumerate() {
                    if idx > 0 {
                        out.push(',');
                    }
                    if matches!(value, Value::Null | Value::Undefined) {
                        continue;
                    }
                    out.push_str(&value.as_string());
                }
                out
            }
            Self::Date(_) => "[object Date]".into(),
            Self::Null => "null".into(),
            Self::Undefined => "undefined".into(),
            Self::Node(node) => format!("node-{}", node.0),
            Self::NodeList(_) => "[object NodeList]".into(),
            Self::FormData(_) => "[object FormData]".into(),
            Self::Function(_) => "[object Function]".into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DomProp {
    Value,
    Checked,
    Readonly,
    Required,
    Disabled,
    TextContent,
    InnerHtml,
    ClassName,
    Id,
    Name,
    OffsetWidth,
    OffsetHeight,
    OffsetLeft,
    OffsetTop,
    ScrollWidth,
    ScrollHeight,
    ScrollLeft,
    ScrollTop,
    Dataset(String),
    Style(String),
    ActiveElement,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DomIndex {
    Static(usize),
    Dynamic(String),
}

impl DomIndex {
    fn describe(&self) -> String {
        match self {
            Self::Static(index) => index.to_string(),
            Self::Dynamic(expr) => expr.clone(),
        }
    }

    fn static_index(&self) -> Option<usize> {
        match self {
            Self::Static(index) => Some(*index),
            Self::Dynamic(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DomQuery {
    DocumentRoot,
    ById(String),
    BySelector(String),
    BySelectorAll {
        selector: String,
    },
    BySelectorAllIndex {
        selector: String,
        index: DomIndex,
    },
    QuerySelector {
        target: Box<DomQuery>,
        selector: String,
    },
    QuerySelectorAll {
        target: Box<DomQuery>,
        selector: String,
    },
    Index {
        target: Box<DomQuery>,
        index: DomIndex,
    },
    QuerySelectorAllIndex {
        target: Box<DomQuery>,
        selector: String,
        index: DomIndex,
    },
    FormElementsIndex {
        form: Box<DomQuery>,
        index: DomIndex,
    },
    Var(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum FormDataSource {
    NewForm(DomQuery),
    Var(String),
}

impl DomQuery {
    fn describe_call(&self) -> String {
        match self {
            Self::DocumentRoot => "document".into(),
            Self::ById(id) => format!("document.getElementById('{id}')"),
            Self::BySelector(selector) => format!("document.querySelector('{selector}')"),
            Self::BySelectorAll { selector } => format!("document.querySelectorAll('{selector}')"),
            Self::BySelectorAllIndex { selector, index } => {
                format!(
                    "document.querySelectorAll('{selector}')[{}]",
                    index.describe()
                )
            }
            Self::QuerySelector { target, selector } => {
                format!("{}.querySelector('{selector}')", target.describe_call())
            }
            Self::QuerySelectorAll { target, selector } => {
                format!("{}.querySelectorAll('{selector}')", target.describe_call())
            }
            Self::Index { target, index } => {
                format!("{}[{}]", target.describe_call(), index.describe())
            }
            Self::QuerySelectorAllIndex {
                target,
                selector,
                index,
            } => {
                format!(
                    "{}.querySelectorAll('{selector}')[{}]",
                    target.describe_call(),
                    index.describe()
                )
            }
            Self::FormElementsIndex { form, index } => {
                format!("{}.elements[{}]", form.describe_call(), index.describe())
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
    BitOr,
    BitXor,
    BitAnd,
    ShiftLeft,
    ShiftRight,
    UnsignedShiftRight,
    Pow,
    Lt,
    Gt,
    Le,
    Ge,
    In,
    InstanceOf,
    Sub,
    Mul,
    Div,
    Mod,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VarAssignOp {
    Assign,
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Mod,
    BitOr,
    BitXor,
    BitAnd,
    ShiftLeft,
    ShiftRight,
    UnsignedShiftRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EventExprProp {
    Type,
    Target,
    CurrentTarget,
    TargetName,
    CurrentTargetName,
    DefaultPrevented,
    IsTrusted,
    Bubbles,
    Cancelable,
    TargetId,
    CurrentTargetId,
    EventPhase,
    TimeStamp,
}

#[derive(Debug, Clone, PartialEq)]
enum Expr {
    String(String),
    Bool(bool),
    Null,
    Undefined,
    Number(i64),
    Float(f64),
    DateNow,
    DateNew {
        value: Option<Box<Expr>>,
    },
    DateParse(Box<Expr>),
    DateUtc {
        args: Vec<Expr>,
    },
    DateGetTime(String),
    DateSetTime {
        target: String,
        value: Box<Expr>,
    },
    DateToIsoString(String),
    DateGetFullYear(String),
    DateGetMonth(String),
    DateGetDate(String),
    DateGetHours(String),
    DateGetMinutes(String),
    DateGetSeconds(String),
    MathRandom,
    EncodeUri(Box<Expr>),
    EncodeUriComponent(Box<Expr>),
    DecodeUri(Box<Expr>),
    DecodeUriComponent(Box<Expr>),
    Escape(Box<Expr>),
    Unescape(Box<Expr>),
    IsNaN(Box<Expr>),
    IsFinite(Box<Expr>),
    Atob(Box<Expr>),
    Btoa(Box<Expr>),
    ParseInt {
        value: Box<Expr>,
        radix: Option<Box<Expr>>,
    },
    ParseFloat(Box<Expr>),
    ArrayLiteral(Vec<Expr>),
    ArrayIsArray(Box<Expr>),
    ArrayLength(String),
    ArrayIndex {
        target: String,
        index: Box<Expr>,
    },
    ArrayPush {
        target: String,
        args: Vec<Expr>,
    },
    ArrayPop(String),
    ArrayShift(String),
    ArrayUnshift {
        target: String,
        args: Vec<Expr>,
    },
    ArrayMap {
        target: String,
        callback: ScriptHandler,
    },
    ArrayFilter {
        target: String,
        callback: ScriptHandler,
    },
    ArrayReduce {
        target: String,
        callback: ScriptHandler,
        initial: Option<Box<Expr>>,
    },
    ArrayForEach {
        target: String,
        callback: ScriptHandler,
    },
    ArrayFind {
        target: String,
        callback: ScriptHandler,
    },
    ArraySome {
        target: String,
        callback: ScriptHandler,
    },
    ArrayEvery {
        target: String,
        callback: ScriptHandler,
    },
    ArrayIncludes {
        target: String,
        search: Box<Expr>,
        from_index: Option<Box<Expr>>,
    },
    ArraySlice {
        target: String,
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
    },
    ArraySplice {
        target: String,
        start: Box<Expr>,
        delete_count: Option<Box<Expr>>,
        items: Vec<Expr>,
    },
    ArrayJoin {
        target: String,
        separator: Option<Box<Expr>>,
    },
    StringTrim {
        value: Box<Expr>,
        mode: StringTrimMode,
    },
    StringToUpperCase(Box<Expr>),
    StringToLowerCase(Box<Expr>),
    StringIncludes {
        value: Box<Expr>,
        search: Box<Expr>,
        position: Option<Box<Expr>>,
    },
    StringStartsWith {
        value: Box<Expr>,
        search: Box<Expr>,
        position: Option<Box<Expr>>,
    },
    StringEndsWith {
        value: Box<Expr>,
        search: Box<Expr>,
        length: Option<Box<Expr>>,
    },
    StringSlice {
        value: Box<Expr>,
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
    },
    StringSubstring {
        value: Box<Expr>,
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
    },
    StringSplit {
        value: Box<Expr>,
        separator: Option<Box<Expr>>,
        limit: Option<Box<Expr>>,
    },
    StringReplace {
        value: Box<Expr>,
        from: Box<Expr>,
        to: Box<Expr>,
    },
    StringIndexOf {
        value: Box<Expr>,
        search: Box<Expr>,
        position: Option<Box<Expr>>,
    },
    Fetch(Box<Expr>),
    Alert(Box<Expr>),
    Confirm(Box<Expr>),
    Prompt {
        message: Box<Expr>,
        default: Option<Box<Expr>>,
    },
    Var(String),
    DomRef(DomQuery),
    CreateElement(String),
    CreateTextNode(String),
    SetTimeout {
        handler: TimerInvocation,
        delay_ms: Box<Expr>,
    },
    SetInterval {
        handler: TimerInvocation,
        delay_ms: Box<Expr>,
    },
    Function {
        handler: ScriptHandler,
    },
    QueueMicrotask {
        handler: ScriptHandler,
    },
    PromiseThen {
        callback: ScriptHandler,
    },
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    DomRead {
        target: DomQuery,
        prop: DomProp,
    },
    DomMatches {
        target: DomQuery,
        selector: String,
    },
    DomClosest {
        target: DomQuery,
        selector: String,
    },
    DomComputedStyleProperty {
        target: DomQuery,
        property: String,
    },
    ClassListContains {
        target: DomQuery,
        class_name: String,
    },
    QuerySelectorAllLength {
        target: DomQuery,
    },
    FormElementsLength {
        form: DomQuery,
    },
    FormDataNew {
        form: DomQuery,
    },
    FormDataGet {
        source: FormDataSource,
        name: String,
    },
    FormDataHas {
        source: FormDataSource,
        name: String,
    },
    FormDataGetAllLength {
        source: FormDataSource,
        name: String,
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
    Neg(Box<Expr>),
    Pos(Box<Expr>),
    BitNot(Box<Expr>),
    Not(Box<Expr>),
    Void(Box<Expr>),
    Delete(Box<Expr>),
    TypeOf(Box<Expr>),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StringTrimMode {
    Both,
    Start,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NodeTreeMethod {
    After,
    Append,
    AppendChild,
    Before,
    ReplaceWith,
    Prepend,
    RemoveChild,
    InsertBefore,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InsertAdjacentPosition {
    BeforeBegin,
    AfterBegin,
    BeforeEnd,
    AfterEnd,
}

#[derive(Debug, Clone, PartialEq)]
enum Stmt {
    VarDecl {
        name: String,
        expr: Expr,
    },
    VarAssign {
        name: String,
        op: VarAssignOp,
        expr: Expr,
    },
    FormDataAppend {
        target_var: String,
        name: Expr,
        value: Expr,
    },
    DomAssign {
        target: DomQuery,
        prop: DomProp,
        expr: Expr,
    },
    ClassListCall {
        target: DomQuery,
        method: ClassListMethod,
        class_names: Vec<String>,
        force: Option<Expr>,
    },
    ClassListForEach {
        target: DomQuery,
        item_var: String,
        index_var: Option<String>,
        body: Vec<Stmt>,
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
    NodeTreeMutation {
        target: DomQuery,
        method: NodeTreeMethod,
        child: Expr,
        reference: Option<Expr>,
    },
    InsertAdjacentElement {
        target: DomQuery,
        position: InsertAdjacentPosition,
        node: Expr,
    },
    InsertAdjacentText {
        target: DomQuery,
        position: InsertAdjacentPosition,
        text: Expr,
    },
    InsertAdjacentHTML {
        target: DomQuery,
        position: Expr,
        html: Expr,
    },
    SetTimeout {
        handler: TimerInvocation,
        delay_ms: Expr,
    },
    SetInterval {
        handler: TimerInvocation,
        delay_ms: Expr,
    },
    QueueMicrotask {
        handler: ScriptHandler,
    },
    ClearTimeout {
        timer_id: Expr,
    },
    NodeRemove {
        target: DomQuery,
    },
    ForEach {
        target: Option<DomQuery>,
        selector: String,
        item_var: String,
        index_var: Option<String>,
        body: Vec<Stmt>,
    },
    ArrayForEach {
        target: String,
        callback: ScriptHandler,
    },
    For {
        init: Option<Box<Stmt>>,
        cond: Option<Expr>,
        post: Option<Box<Stmt>>,
        body: Vec<Stmt>,
    },
    ForIn {
        item_var: String,
        iterable: Expr,
        body: Vec<Stmt>,
    },
    ForOf {
        item_var: String,
        iterable: Expr,
        body: Vec<Stmt>,
    },
    DoWhile {
        cond: Expr,
        body: Vec<Stmt>,
    },
    Break,
    Continue,
    While {
        cond: Expr,
        body: Vec<Stmt>,
    },
    Return {
        value: Option<Expr>,
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
    DomMethodCall {
        target: DomQuery,
        method: DomMethod,
    },
    Expr(Expr),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExecFlow {
    Continue,
    Break,
    ContinueLoop,
    Return,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DomMethod {
    Focus,
    Blur,
    Click,
    ScrollIntoView,
    Submit,
    Reset,
}

#[derive(Debug, Clone, PartialEq)]
struct ScriptHandler {
    params: Vec<String>,
    stmts: Vec<Stmt>,
}

impl ScriptHandler {
    fn first_event_param(&self) -> Option<&str> {
        self.params.first().map(String::as_str)
    }

    fn bind_event_params(&self, args: &[Value], env: &mut HashMap<String, Value>) {
        for (index, name) in self.params.iter().enumerate() {
            let value = args.get(index).cloned().unwrap_or(Value::Undefined);
            env.insert(name.clone(), value);
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum TimerCallback {
    Inline(ScriptHandler),
    Reference(String),
}

#[derive(Debug, Clone, PartialEq)]
struct TimerInvocation {
    callback: TimerCallback,
    args: Vec<Expr>,
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
    event_phase: i32,
    time_stamp_ms: i64,
    default_prevented: bool,
    is_trusted: bool,
    bubbles: bool,
    cancelable: bool,
    propagation_stopped: bool,
    immediate_propagation_stopped: bool,
}

impl EventState {
    fn new(event_type: &str, target: NodeId, time_stamp_ms: i64) -> Self {
        Self {
            event_type: event_type.to_string(),
            target,
            current_target: target,
            event_phase: 2,
            time_stamp_ms,
            default_prevented: false,
            is_trusted: true,
            bubbles: true,
            cancelable: true,
            propagation_stopped: false,
            immediate_propagation_stopped: false,
        }
    }

    fn new_untrusted(event_type: &str, target: NodeId, time_stamp_ms: i64) -> Self {
        let mut event = Self::new(event_type, target, time_stamp_ms);
        event.is_trusted = false;
        event.bubbles = false;
        event.cancelable = false;
        event
    }
}

#[derive(Debug, Clone)]
struct ParseOutput {
    dom: Dom,
    scripts: Vec<String>,
}

#[derive(Debug, Clone)]
struct ScheduledTask {
    id: i64,
    due_at: i64,
    order: i64,
    interval_ms: Option<i64>,
    callback: TimerCallback,
    callback_args: Vec<Value>,
    env: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
struct ScheduledMicrotask {
    handler: ScriptHandler,
    env: HashMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingTimer {
    pub id: i64,
    pub due_at: i64,
    pub order: i64,
    pub interval_ms: Option<i64>,
}

#[derive(Debug)]
pub struct Harness {
    dom: Dom,
    listeners: ListenerStore,
    script_env: HashMap<String, Value>,
    task_queue: Vec<ScheduledTask>,
    microtask_queue: VecDeque<ScheduledMicrotask>,
    active_element: Option<NodeId>,
    now_ms: i64,
    timer_step_limit: usize,
    next_timer_id: i64,
    next_task_order: i64,
    task_depth: usize,
    running_timer_id: Option<i64>,
    running_timer_canceled: bool,
    rng_state: u64,
    fetch_mocks: HashMap<String, String>,
    fetch_calls: Vec<String>,
    alert_messages: Vec<String>,
    confirm_responses: VecDeque<bool>,
    default_confirm_response: bool,
    prompt_responses: VecDeque<Option<String>>,
    default_prompt_response: Option<String>,
    trace: bool,
    trace_events: bool,
    trace_timers: bool,
    trace_logs: Vec<String>,
    trace_log_limit: usize,
    trace_to_stderr: bool,
}

#[derive(Debug)]
pub struct MockWindow {
    pages: Vec<MockPage>,
    current: usize,
}

#[derive(Debug)]
pub struct MockPage {
    pub url: String,
    harness: Harness,
}

impl MockWindow {
    pub fn new() -> Self {
        Self {
            pages: Vec::new(),
            current: 0,
        }
    }

    pub fn open_page(&mut self, url: &str, html: &str) -> Result<usize> {
        let harness = Harness::from_html(html)?;
        if let Some(index) = self
            .pages
            .iter()
            .position(|page| page.url.eq_ignore_ascii_case(url))
        {
            self.pages[index] = MockPage {
                url: url.to_string(),
                harness,
            };
            self.current = index;
            Ok(index)
        } else {
            self.pages.push(MockPage {
                url: url.to_string(),
                harness,
            });
            self.current = self.pages.len() - 1;
            Ok(self.current)
        }
    }

    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    pub fn switch_to(&mut self, url: &str) -> Result<()> {
        let index = self
            .pages
            .iter()
            .position(|page| page.url == url)
            .ok_or_else(|| Error::ScriptRuntime(format!("unknown page: {url}")))?;
        self.current = index;
        Ok(())
    }

    pub fn switch_to_index(&mut self, index: usize) -> Result<()> {
        if index >= self.pages.len() {
            return Err(Error::ScriptRuntime(format!(
                "page index out of range: {index}"
            )));
        }
        self.current = index;
        Ok(())
    }

    pub fn current_url(&self) -> Result<&str> {
        self.pages
            .get(self.current)
            .map(|page| page.url.as_str())
            .ok_or_else(|| Error::ScriptRuntime("window has no pages".into()))
    }

    pub fn current_document_mut(&mut self) -> Result<&mut Harness> {
        self.pages
            .get_mut(self.current)
            .map(|page| &mut page.harness)
            .ok_or_else(|| Error::ScriptRuntime("window has no pages".into()))
    }

    pub fn current_document(&self) -> Result<&Harness> {
        self.pages
            .get(self.current)
            .map(|page| &page.harness)
            .ok_or_else(|| Error::ScriptRuntime("window has no pages".into()))
    }

    pub fn with_current_document<R>(
        &mut self,
        f: impl FnOnce(&mut Harness) -> Result<R>,
    ) -> Result<R> {
        let harness = self.current_document_mut()?;
        f(harness)
    }

    pub fn type_text(&mut self, selector: &str, text: &str) -> Result<()> {
        let page = self.current_document_mut()?;
        page.type_text(selector, text)
    }

    pub fn set_checked(&mut self, selector: &str, checked: bool) -> Result<()> {
        let page = self.current_document_mut()?;
        page.set_checked(selector, checked)
    }

    pub fn click(&mut self, selector: &str) -> Result<()> {
        let page = self.current_document_mut()?;
        page.click(selector)
    }

    pub fn submit(&mut self, selector: &str) -> Result<()> {
        let page = self.current_document_mut()?;
        page.submit(selector)
    }

    pub fn dispatch(&mut self, selector: &str, event: &str) -> Result<()> {
        let page = self.current_document_mut()?;
        page.dispatch(selector, event)
    }

    pub fn assert_text(&mut self, selector: &str, expected: &str) -> Result<()> {
        let page = self.current_document_mut()?;
        page.assert_text(selector, expected)
    }

    pub fn assert_value(&mut self, selector: &str, expected: &str) -> Result<()> {
        let page = self.current_document_mut()?;
        page.assert_value(selector, expected)
    }

    pub fn assert_checked(&mut self, selector: &str, expected: bool) -> Result<()> {
        let page = self.current_document_mut()?;
        page.assert_checked(selector, expected)
    }

    pub fn assert_exists(&mut self, selector: &str) -> Result<()> {
        let page = self.current_document_mut()?;
        page.assert_exists(selector)
    }

    pub fn take_trace_logs(&mut self) -> Result<Vec<String>> {
        let page = self.current_document_mut()?;
        Ok(page.take_trace_logs())
    }
}

impl MockPage {
    pub fn url(&self) -> &str {
        self.url.as_str()
    }

    pub fn harness(&self) -> &Harness {
        &self.harness
    }

    pub fn harness_mut(&mut self) -> &mut Harness {
        &mut self.harness
    }
}

impl Harness {
    pub fn from_html(html: &str) -> Result<Self> {
        let ParseOutput { dom, scripts } = parse_html(html)?;
        let mut harness = Self {
            dom,
            listeners: ListenerStore::default(),
            script_env: HashMap::new(),
            task_queue: Vec::new(),
            microtask_queue: VecDeque::new(),
            active_element: None,
            now_ms: 0,
            timer_step_limit: 10_000,
            next_timer_id: 1,
            next_task_order: 0,
            task_depth: 0,
            running_timer_id: None,
            running_timer_canceled: false,
            rng_state: 0x9E37_79B9_7F4A_7C15,
            fetch_mocks: HashMap::new(),
            fetch_calls: Vec::new(),
            alert_messages: Vec::new(),
            confirm_responses: VecDeque::new(),
            default_confirm_response: false,
            prompt_responses: VecDeque::new(),
            default_prompt_response: None,
            trace: false,
            trace_events: true,
            trace_timers: true,
            trace_logs: Vec::new(),
            trace_log_limit: 10_000,
            trace_to_stderr: true,
        };

        for script in scripts {
            harness.compile_and_register_script(&script)?;
        }

        Ok(harness)
    }

    pub fn enable_trace(&mut self, enabled: bool) {
        self.trace = enabled;
    }

    pub fn take_trace_logs(&mut self) -> Vec<String> {
        std::mem::take(&mut self.trace_logs)
    }

    pub fn set_trace_stderr(&mut self, enabled: bool) {
        self.trace_to_stderr = enabled;
    }

    pub fn set_trace_events(&mut self, enabled: bool) {
        self.trace_events = enabled;
    }

    pub fn set_trace_timers(&mut self, enabled: bool) {
        self.trace_timers = enabled;
    }

    pub fn set_trace_log_limit(&mut self, max_entries: usize) -> Result<()> {
        if max_entries == 0 {
            return Err(Error::ScriptRuntime(
                "set_trace_log_limit requires at least 1 entry".into(),
            ));
        }
        self.trace_log_limit = max_entries;
        while self.trace_logs.len() > self.trace_log_limit {
            self.trace_logs.remove(0);
        }
        Ok(())
    }

    pub fn set_random_seed(&mut self, seed: u64) {
        self.rng_state = if seed == 0 {
            0xA5A5_A5A5_A5A5_A5A5
        } else {
            seed
        };
    }

    pub fn set_fetch_mock(&mut self, url: &str, body: &str) {
        self.fetch_mocks.insert(url.to_string(), body.to_string());
    }

    pub fn clear_fetch_mocks(&mut self) {
        self.fetch_mocks.clear();
    }

    pub fn take_fetch_calls(&mut self) -> Vec<String> {
        std::mem::take(&mut self.fetch_calls)
    }

    pub fn enqueue_confirm_response(&mut self, accepted: bool) {
        self.confirm_responses.push_back(accepted);
    }

    pub fn set_default_confirm_response(&mut self, accepted: bool) {
        self.default_confirm_response = accepted;
    }

    pub fn enqueue_prompt_response(&mut self, value: Option<&str>) {
        self.prompt_responses
            .push_back(value.map(std::string::ToString::to_string));
    }

    pub fn set_default_prompt_response(&mut self, value: Option<&str>) {
        self.default_prompt_response = value.map(std::string::ToString::to_string);
    }

    pub fn take_alert_messages(&mut self) -> Vec<String> {
        std::mem::take(&mut self.alert_messages)
    }

    pub fn set_timer_step_limit(&mut self, max_steps: usize) -> Result<()> {
        if max_steps == 0 {
            return Err(Error::ScriptRuntime(
                "set_timer_step_limit requires at least 1 step".into(),
            ));
        }
        self.timer_step_limit = max_steps;
        Ok(())
    }

    pub fn type_text(&mut self, selector: &str, text: &str) -> Result<()> {
        let target = self.select_one(selector)?;
        if self.dom.disabled(target) {
            return Ok(());
        }
        if self.dom.readonly(target) {
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
        self.click_node(target)
    }

    fn click_node_with_env(
        &mut self,
        target: NodeId,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        if self.dom.disabled(target) {
            return Ok(());
        }

        self.dom.set_active_pseudo_element(Some(target));
        let result: Result<()> = (|| {
            let click_outcome = self.dispatch_event_with_env(target, "click", env, true)?;
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
                    self.dispatch_event_with_env(form_id, "submit", env, true)?;
                }
            }

            Ok(())
        })();
        self.dom.set_active_pseudo_element(None);
        result
    }

    fn click_node(&mut self, target: NodeId) -> Result<()> {
        let mut env = self.script_env.clone();
        self.click_node_with_env(target, &mut env)
    }

    pub fn focus(&mut self, selector: &str) -> Result<()> {
        let target = self.select_one(selector)?;
        self.focus_node(target)
    }

    pub fn blur(&mut self, selector: &str) -> Result<()> {
        let target = self.select_one(selector)?;
        self.blur_node(target)
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

    fn submit_form_with_env(
        &mut self,
        target: NodeId,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
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
            self.dispatch_event_with_env(form_id, "submit", env, true)?;
        }

        Ok(())
    }

    fn reset_form_with_env(
        &mut self,
        target: NodeId,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        let Some(form_id) = self.resolve_form_for_submit(target) else {
            return Ok(());
        };

        let outcome = self.dispatch_event_with_env(form_id, "reset", env, true)?;
        if outcome.default_prevented {
            return Ok(());
        }

        let controls = self.form_elements(form_id)?;
        for control in controls {
            if is_checkbox_input(&self.dom, control) || is_radio_input(&self.dom, control) {
                let default_checked = self.dom.attr(control, "checked").is_some();
                self.dom.set_checked(control, default_checked)?;
                continue;
            }

            if self
                .dom
                .tag_name(control)
                .map(|tag| tag.eq_ignore_ascii_case("select"))
                .unwrap_or(false)
            {
                self.dom.sync_select_value(control)?;
                continue;
            }

            let default_value = self.dom.attr(control, "value").unwrap_or_default();
            self.dom.set_value(control, &default_value)?;
        }

        Ok(())
    }

    pub fn dispatch(&mut self, selector: &str, event: &str) -> Result<()> {
        let target = self.select_one(selector)?;
        let mut env = self.script_env.clone();
        let _ = self.dispatch_event_with_env(target, event, &mut env, false)?;
        self.script_env = env;
        Ok(())
    }

    pub fn now_ms(&self) -> i64 {
        self.now_ms
    }

    pub fn clear_timer(&mut self, timer_id: i64) -> bool {
        let existed = self.running_timer_id == Some(timer_id)
            || self.task_queue.iter().any(|task| task.id == timer_id);
        self.clear_timeout(timer_id);
        existed
    }

    pub fn clear_all_timers(&mut self) -> usize {
        let cleared = self.task_queue.len();
        self.task_queue.clear();
        if self.running_timer_id.is_some() {
            self.running_timer_canceled = true;
        }
        self.trace_timer_line(format!("[timer] clear_all cleared={cleared}"));
        cleared
    }

    pub fn pending_timers(&self) -> Vec<PendingTimer> {
        let mut timers = self
            .task_queue
            .iter()
            .map(|task| PendingTimer {
                id: task.id,
                due_at: task.due_at,
                order: task.order,
                interval_ms: task.interval_ms,
            })
            .collect::<Vec<_>>();
        timers.sort_by_key(|timer| (timer.due_at, timer.order));
        timers
    }

    pub fn advance_time(&mut self, delta_ms: i64) -> Result<()> {
        if delta_ms < 0 {
            return Err(Error::ScriptRuntime(
                "advance_time requires non-negative milliseconds".into(),
            ));
        }
        let from = self.now_ms;
        self.now_ms = self.now_ms.saturating_add(delta_ms);
        let ran = self.run_due_timers_internal()?;
        self.trace_timer_line(format!(
            "[timer] advance delta_ms={} from={} to={} ran_due={}",
            delta_ms, from, self.now_ms, ran
        ));
        Ok(())
    }

    pub fn advance_time_to(&mut self, target_ms: i64) -> Result<()> {
        if target_ms < self.now_ms {
            return Err(Error::ScriptRuntime(format!(
                "advance_time_to requires target >= now_ms (target={target_ms}, now_ms={})",
                self.now_ms
            )));
        }
        let from = self.now_ms;
        self.now_ms = target_ms;
        let ran = self.run_due_timers_internal()?;
        self.trace_timer_line(format!(
            "[timer] advance_to from={} to={} ran_due={}",
            from, self.now_ms, ran
        ));
        Ok(())
    }

    pub fn flush(&mut self) -> Result<()> {
        let from = self.now_ms;
        let ran = self.run_timer_queue(None, true)?;
        self.trace_timer_line(format!(
            "[timer] flush from={} to={} ran={}",
            from, self.now_ms, ran
        ));
        Ok(())
    }

    pub fn run_next_timer(&mut self) -> Result<bool> {
        let Some(next_idx) = self.next_task_index(None) else {
            self.trace_timer_line("[timer] run_next none".into());
            return Ok(false);
        };

        let task = self.task_queue.remove(next_idx);
        if task.due_at > self.now_ms {
            self.now_ms = task.due_at;
        }
        self.execute_timer_task(task)?;
        Ok(true)
    }

    pub fn run_next_due_timer(&mut self) -> Result<bool> {
        let Some(next_idx) = self.next_task_index(Some(self.now_ms)) else {
            self.trace_timer_line("[timer] run_next_due none".into());
            return Ok(false);
        };

        let task = self.task_queue.remove(next_idx);
        self.execute_timer_task(task)?;
        Ok(true)
    }

    pub fn run_due_timers(&mut self) -> Result<usize> {
        let ran = self.run_due_timers_internal()?;
        self.trace_timer_line(format!(
            "[timer] run_due now_ms={} ran={}",
            self.now_ms, ran
        ));
        Ok(ran)
    }

    fn run_due_timers_internal(&mut self) -> Result<usize> {
        self.run_timer_queue(Some(self.now_ms), false)
    }

    fn run_timer_queue(&mut self, due_limit: Option<i64>, advance_clock: bool) -> Result<usize> {
        let mut steps = 0usize;
        while let Some(next_idx) = self.next_task_index(due_limit) {
            steps += 1;
            if steps > self.timer_step_limit {
                return Err(self.timer_step_limit_error(self.timer_step_limit, steps, due_limit));
            }
            let task = self.task_queue.remove(next_idx);
            if advance_clock && task.due_at > self.now_ms {
                self.now_ms = task.due_at;
            }
            self.execute_timer_task(task)?;
        }
        Ok(steps)
    }

    fn timer_step_limit_error(
        &self,
        max_steps: usize,
        steps: usize,
        due_limit: Option<i64>,
    ) -> Error {
        let due_limit_desc = due_limit
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".into());

        let next_task_desc = self
            .next_task_index(due_limit)
            .and_then(|idx| self.task_queue.get(idx))
            .map(|task| {
                let interval_desc = task
                    .interval_ms
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "none".into());
                format!(
                    "id={},due_at={},order={},interval_ms={}",
                    task.id, task.due_at, task.order, interval_desc
                )
            })
            .unwrap_or_else(|| "none".into());

        Error::ScriptRuntime(format!(
            "flush exceeded max task steps (possible uncleared setInterval): limit={max_steps}, steps={steps}, now_ms={}, due_limit={}, pending_tasks={}, next_task={}",
            self.now_ms,
            due_limit_desc,
            self.task_queue.len(),
            next_task_desc
        ))
    }

    fn next_task_index(&self, due_limit: Option<i64>) -> Option<usize> {
        self.task_queue
            .iter()
            .enumerate()
            .filter(|(_, task)| {
                if let Some(limit) = due_limit {
                    task.due_at <= limit
                } else {
                    true
                }
            })
            .min_by_key(|(_, task)| (task.due_at, task.order))
            .map(|(idx, _)| idx)
    }

    fn execute_timer_task(&mut self, mut task: ScheduledTask) -> Result<()> {
        let interval_desc = task
            .interval_ms
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".into());
        self.trace_timer_line(format!(
            "[timer] run id={} due_at={} interval_ms={} now_ms={}",
            task.id, task.due_at, interval_desc, self.now_ms
        ));

        self.running_timer_id = Some(task.id);
        self.running_timer_canceled = false;
        let mut event = EventState::new("timeout", self.dom.root, self.now_ms);
        self.run_in_task_context(|this| {
            this.execute_timer_task_callback(&task.callback, &task.callback_args, &mut event, &mut task.env)
                .map(|_| ())
        })?;
        let canceled = self.running_timer_canceled;
        self.running_timer_id = None;
        self.running_timer_canceled = false;

        if let Some(interval_ms) = task.interval_ms {
            if !canceled {
                let delay_ms = interval_ms.max(0);
                let due_at = task.due_at.saturating_add(delay_ms);
                let order = self.next_task_order;
                self.next_task_order += 1;
                self.task_queue.push(ScheduledTask {
                    id: task.id,
                    due_at,
                    order,
                    interval_ms: Some(delay_ms),
                    callback: task.callback,
                    callback_args: task.callback_args,
                    env: task.env,
                });
                self.trace_timer_line(format!(
                    "[timer] requeue id={} due_at={} interval_ms={}",
                    task.id, due_at, delay_ms
                ));
            }
        }

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

    fn form_elements(&self, form: NodeId) -> Result<Vec<NodeId>> {
        let tag = self
            .dom
            .tag_name(form)
            .ok_or_else(|| Error::ScriptRuntime("elements target is not an element".into()))?;
        if !tag.eq_ignore_ascii_case("form") {
            return Err(Error::ScriptRuntime(format!(
                "{}.elements target is not a form",
                self.event_node_label(form)
            )));
        }

        let mut out = Vec::new();
        self.collect_form_controls(form, &mut out);
        Ok(out)
    }

    fn form_data_entries(&self, form: NodeId) -> Result<Vec<(String, String)>> {
        let mut out = Vec::new();
        for control in self.form_elements(form)? {
            if !self.is_successful_form_data_control(control)? {
                continue;
            }
            let name = self.dom.attr(control, "name").unwrap_or_default();
            let value = self.form_data_control_value(control)?;
            out.push((name, value));
        }
        Ok(out)
    }

    fn is_successful_form_data_control(&self, control: NodeId) -> Result<bool> {
        if self.dom.disabled(control) {
            return Ok(false);
        }
        let name = self.dom.attr(control, "name").unwrap_or_default();
        if name.is_empty() {
            return Ok(false);
        }

        let tag = self
            .dom
            .tag_name(control)
            .ok_or_else(|| Error::ScriptRuntime("FormData target is not an element".into()))?;

        if tag.eq_ignore_ascii_case("button") {
            return Ok(false);
        }

        if tag.eq_ignore_ascii_case("input") {
            let kind = self
                .dom
                .attr(control, "type")
                .unwrap_or_default()
                .to_ascii_lowercase();
            if matches!(
                kind.as_str(),
                "button" | "submit" | "reset" | "file" | "image"
            ) {
                return Ok(false);
            }
            if kind == "checkbox" || kind == "radio" {
                return self.dom.checked(control);
            }
        }

        Ok(true)
    }

    fn form_data_control_value(&self, control: NodeId) -> Result<String> {
        let mut value = self.dom.value(control)?;
        if value.is_empty()
            && (is_checkbox_input(&self.dom, control) || is_radio_input(&self.dom, control))
        {
            value = "on".into();
        }
        Ok(value)
    }

    fn eval_form_data_source(
        &mut self,
        source: &FormDataSource,
        env: &HashMap<String, Value>,
    ) -> Result<Vec<(String, String)>> {
        match source {
            FormDataSource::NewForm(form) => {
                let form_node = self.resolve_dom_query_required_runtime(form, env)?;
                self.form_data_entries(form_node)
            }
            FormDataSource::Var(name) => match env.get(name) {
                Some(Value::FormData(entries)) => Ok(entries.clone()),
                Some(_) => Err(Error::ScriptRuntime(format!(
                    "variable '{}' is not a FormData instance",
                    name
                ))),
                None => Err(Error::ScriptRuntime(format!(
                    "unknown FormData variable: {}",
                    name
                ))),
            },
        }
    }

    fn collect_form_controls(&self, node: NodeId, out: &mut Vec<NodeId>) {
        for child in &self.dom.nodes[node.0].children {
            if is_form_control(&self.dom, *child) {
                out.push(*child);
            }
            self.collect_form_controls(*child, out);
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
        let mut env = self.script_env.clone();
        let event = self.dispatch_event_with_env(target, event_type, &mut env, true)?;
        self.script_env = env;
        Ok(event)
    }

    fn dispatch_event_with_env(
        &mut self,
        target: NodeId,
        event_type: &str,
        env: &mut HashMap<String, Value>,
        trusted: bool,
    ) -> Result<EventState> {
        let mut event = if trusted {
            EventState::new(event_type, target, self.now_ms)
        } else {
            EventState::new_untrusted(event_type, target, self.now_ms)
        };
        self.run_in_task_context(|this| {
            let mut path = Vec::new();
            let mut cursor = Some(target);
            while let Some(node) = cursor {
                path.push(node);
                cursor = this.dom.parent(node);
            }
            path.reverse();

            if path.is_empty() {
                this.trace_event_done(&event, "empty_path");
                return Ok(());
            }

            // Capture phase.
            if path.len() >= 2 {
                for node in &path[..path.len() - 1] {
                    event.event_phase = 1;
                    event.current_target = *node;
                    this.invoke_listeners(*node, &mut event, env, true)?;
                    if event.propagation_stopped {
                        this.trace_event_done(&event, "propagation_stopped");
                        return Ok(());
                    }
                }
            }

            // Target phase: capture listeners first.
            event.event_phase = 2;
            event.current_target = target;
            this.invoke_listeners(target, &mut event, env, true)?;
            if event.propagation_stopped {
                this.trace_event_done(&event, "propagation_stopped");
                return Ok(());
            }

            // Target phase: bubble listeners.
            event.event_phase = 2;
            this.invoke_listeners(target, &mut event, env, false)?;
            if event.propagation_stopped {
                this.trace_event_done(&event, "propagation_stopped");
                return Ok(());
            }

            // Bubble phase.
            if path.len() >= 2 {
                for node in path[..path.len() - 1].iter().rev() {
                    event.event_phase = 3;
                    event.current_target = *node;
                    this.invoke_listeners(*node, &mut event, env, false)?;
                    if event.propagation_stopped {
                        this.trace_event_done(&event, "propagation_stopped");
                        return Ok(());
                    }
                }
            }

            this.trace_event_done(&event, "completed");
            Ok(())
        })?;
        Ok(event)
    }

    fn focus_node(&mut self, node: NodeId) -> Result<()> {
        let mut env = self.script_env.clone();
        self.focus_node_with_env(node, &mut env)?;
        self.script_env = env;
        Ok(())
    }

    fn focus_node_with_env(
        &mut self,
        node: NodeId,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        if self.dom.disabled(node) {
            return Ok(());
        }

        if self.active_element == Some(node) {
            return Ok(());
        }

        if let Some(current) = self.active_element {
            self.blur_node_with_env(current, env)?;
        }

        self.active_element = Some(node);
        self.dom.set_active_element(Some(node));
        self.dispatch_event_with_env(node, "focusin", env, true)?;
        self.dispatch_event_with_env(node, "focus", env, true)?;
        Ok(())
    }

    fn blur_node(&mut self, node: NodeId) -> Result<()> {
        let mut env = self.script_env.clone();
        self.blur_node_with_env(node, &mut env)?;
        self.script_env = env;
        Ok(())
    }

    fn blur_node_with_env(&mut self, node: NodeId, env: &mut HashMap<String, Value>) -> Result<()> {
        if self.active_element != Some(node) {
            return Ok(());
        }

        self.dispatch_event_with_env(node, "focusout", env, true)?;
        self.dispatch_event_with_env(node, "blur", env, true)?;
        self.active_element = None;
        self.dom.set_active_element(None);
        Ok(())
    }

    fn scroll_into_view_node_with_env(
        &mut self,
        _node: NodeId,
        _env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        Ok(())
    }

    fn invoke_listeners(
        &mut self,
        node_id: NodeId,
        event: &mut EventState,
        env: &mut HashMap<String, Value>,
        capture: bool,
    ) -> Result<()> {
        let listeners = self.listeners.get(node_id, &event.event_type, capture);
        for listener in listeners {
            if self.trace {
                let phase = if capture { "capture" } else { "bubble" };
                let target_label = self.trace_node_label(event.target);
                let current_label = self.trace_node_label(event.current_target);
                self.trace_event_line(format!(
                    "[event] {} target={} current={} phase={} default_prevented={}",
                    event.event_type, target_label, current_label, phase, event.default_prevented
                ));
            }
            self.execute_handler(&listener.handler, event, env)?;
            if event.immediate_propagation_stopped {
                break;
            }
        }
        Ok(())
    }

    fn trace_event_done(&mut self, event: &EventState, outcome: &str) {
        let target_label = self.trace_node_label(event.target);
        let current_label = self.trace_node_label(event.current_target);
        self.trace_event_line(format!(
            "[event] done {} target={} current={} outcome={} default_prevented={} propagation_stopped={} immediate_stopped={}",
            event.event_type,
            target_label,
            current_label,
            outcome,
            event.default_prevented,
            event.propagation_stopped,
            event.immediate_propagation_stopped
        ));
    }

    fn trace_event_line(&mut self, line: String) {
        if self.trace && self.trace_events {
            self.trace_line(line);
        }
    }

    fn trace_timer_line(&mut self, line: String) {
        if self.trace && self.trace_timers {
            self.trace_line(line);
        }
    }

    fn trace_line(&mut self, line: String) {
        if self.trace {
            if self.trace_to_stderr {
                eprintln!("{line}");
            }
            if self.trace_logs.len() >= self.trace_log_limit {
                self.trace_logs.remove(0);
            }
            self.trace_logs.push(line);
        }
    }

    fn queue_microtask(&mut self, handler: ScriptHandler, env: &HashMap<String, Value>) {
        self.microtask_queue.push_back(ScheduledMicrotask {
            handler,
            env: env.clone(),
        });
    }

    fn run_microtask_queue(&mut self) -> Result<usize> {
        let mut steps = 0usize;
        self.task_depth += 1;
        let result = loop {
            let Some(mut task) = self.microtask_queue.pop_front() else {
                break Ok(());
            };
            steps += 1;
            if steps > self.timer_step_limit {
                break Err(self.timer_step_limit_error(
                    self.timer_step_limit,
                    steps,
                    Some(self.now_ms),
                ));
            }

            let mut event = EventState::new("microtask", self.dom.root, self.now_ms);
            let event_param = task
                .handler
                .first_event_param()
                .map(|event_param| event_param.to_string());
            let run = self.execute_stmts(
                &task.handler.stmts,
                &event_param,
                &mut event,
                &mut task.env,
            );
            let run = run.map(|_| ());
            if let Err(err) = run {
                break Err(err);
            }
        };
        self.task_depth -= 1;
        result?;
        Ok(steps)
    }

    fn run_in_task_context<T>(&mut self, mut run: impl FnMut(&mut Self) -> Result<T>) -> Result<T> {
        self.task_depth += 1;
        let result = run(self);
        self.task_depth -= 1;
        let should_flush_microtasks = self.task_depth == 0;
        match result {
            Ok(value) => {
                if should_flush_microtasks {
                    self.run_microtask_queue()?;
                }
                Ok(value)
            }
            Err(err) => Err(err),
        }
    }

    fn execute_handler(
        &mut self,
        handler: &ScriptHandler,
        event: &mut EventState,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        let event_param = handler
            .first_event_param()
            .map(|event_param| event_param.to_string());
        let flow = self.execute_stmts(&handler.stmts, &event_param, event, env)?;
        env.remove(INTERNAL_RETURN_SLOT);
        match flow {
            ExecFlow::Continue => Ok(()),
            ExecFlow::Break => Err(Error::ScriptRuntime("break statement outside of loop".into())),
            ExecFlow::ContinueLoop => {
                Err(Error::ScriptRuntime("continue statement outside of loop".into()))
            }
            ExecFlow::Return => Ok(()),
        }
    }

    fn execute_timer_task_callback(
        &mut self,
        callback: &TimerCallback,
        callback_args: &[Value],
        event: &mut EventState,
        env: &mut HashMap<String, Value>,
    ) -> Result<()> {
        let handler = match callback {
            TimerCallback::Inline(handler) => handler.clone(),
            TimerCallback::Reference(name) => {
                let value = env
                    .get(name)
                    .cloned()
                    .ok_or_else(|| Error::ScriptRuntime(format!("unknown variable: {name}")))?;
                let Value::Function(handler) = value else {
                    return Err(Error::ScriptRuntime(format!(
                        "timer callback '{name}' is not a function"
                    )));
                };
                handler
            }
        };
        handler.bind_event_params(callback_args, env);

        let event_param = handler
            .first_event_param()
            .map(|event_param| event_param.to_string());
        let flow = self.execute_stmts(&handler.stmts, &event_param, event, env)?;
        env.remove(INTERNAL_RETURN_SLOT);
        match flow {
            ExecFlow::Continue => Ok(()),
            ExecFlow::Break => Err(Error::ScriptRuntime("break statement outside of loop".into())),
            ExecFlow::ContinueLoop => {
                Err(Error::ScriptRuntime("continue statement outside of loop".into()))
            }
            ExecFlow::Return => Ok(()),
        }
    }

    fn execute_stmts(
        &mut self,
        stmts: &[Stmt],
        event_param: &Option<String>,
        event: &mut EventState,
        env: &mut HashMap<String, Value>,
    ) -> Result<ExecFlow> {
        for stmt in stmts {
            match stmt {
                Stmt::VarDecl { name, expr } => {
                    let value = self.eval_expr(expr, env, event_param, event)?;
                    env.insert(name.clone(), value.clone());
                    self.bind_timer_id_to_task_env(name, expr, &value);
                }
                Stmt::VarAssign { name, op, expr } => {
                    let value = self.eval_expr(expr, env, event_param, event)?;
                    let previous = env
                        .get(name)
                        .cloned()
                        .ok_or_else(|| Error::ScriptRuntime(format!("unknown variable: {name}")))?;

                    let next = match op {
                        VarAssignOp::Assign => value,
                        VarAssignOp::Add => self.add_values(&previous, &value),
                        VarAssignOp::Sub => Value::Float(self.numeric_value(&previous) - self.numeric_value(&value)),
                        VarAssignOp::Mul => Value::Float(self.numeric_value(&previous) * self.numeric_value(&value)),
                        VarAssignOp::Pow => Value::Float(
                            self.numeric_value(&previous).powf(self.numeric_value(&value)),
                        ),
                        VarAssignOp::BitOr => self.eval_binary(&BinaryOp::BitOr, &previous, &value)?,
                        VarAssignOp::BitXor => self.eval_binary(&BinaryOp::BitXor, &previous, &value)?,
                        VarAssignOp::BitAnd => self.eval_binary(&BinaryOp::BitAnd, &previous, &value)?,
                        VarAssignOp::ShiftLeft => {
                            self.eval_binary(&BinaryOp::ShiftLeft, &previous, &value)?
                        }
                        VarAssignOp::ShiftRight => {
                            self.eval_binary(&BinaryOp::ShiftRight, &previous, &value)?
                        }
                        VarAssignOp::UnsignedShiftRight => {
                            self.eval_binary(&BinaryOp::UnsignedShiftRight, &previous, &value)?
                        }
                        VarAssignOp::Div => {
                            let rhs = self.numeric_value(&value);
                            if rhs == 0.0 {
                                return Err(Error::ScriptRuntime("division by zero".into()));
                            }
                            Value::Float(self.numeric_value(&previous) / rhs)
                        }
                        VarAssignOp::Mod => {
                            let rhs = self.numeric_value(&value);
                            if rhs == 0.0 {
                                return Err(Error::ScriptRuntime("modulo by zero".into()));
                            }
                            Value::Float(self.numeric_value(&previous) % rhs)
                        }
                    };
                    env.insert(name.clone(), next.clone());
                    self.bind_timer_id_to_task_env(name, expr, &next);
                }
                Stmt::FormDataAppend {
                    target_var,
                    name,
                    value,
                } => {
                    let name = self.eval_expr(name, env, event_param, event)?;
                    let value = self.eval_expr(value, env, event_param, event)?;
                    let entries = env.get_mut(target_var).ok_or_else(|| {
                        Error::ScriptRuntime(format!("unknown FormData variable: {}", target_var))
                    })?;
                    let Value::FormData(entries) = entries else {
                        return Err(Error::ScriptRuntime(format!(
                            "variable '{}' is not a FormData instance",
                            target_var
                        )));
                    };
                    entries.push((name.as_string(), value.as_string()));
                }
                Stmt::DomAssign { target, prop, expr } => {
                    let value = self.eval_expr(expr, env, event_param, event)?;
                    let node = self.resolve_dom_query_required_runtime(target, env)?;
                    match prop {
                        DomProp::TextContent => {
                            self.dom.set_text_content(node, &value.as_string())?
                        }
                        DomProp::InnerHtml => self.dom.set_inner_html(node, &value.as_string())?,
                        DomProp::Value => self.dom.set_value(node, &value.as_string())?,
                        DomProp::Checked => self.dom.set_checked(node, value.truthy())?,
                        DomProp::Readonly => {
                            if value.truthy() {
                                self.dom.set_attr(node, "readonly", "true")?;
                            } else {
                                self.dom.remove_attr(node, "readonly")?;
                            }
                        }
                        DomProp::Required => {
                            if value.truthy() {
                                self.dom.set_attr(node, "required", "true")?;
                            } else {
                                self.dom.remove_attr(node, "required")?;
                            }
                        }
                        DomProp::Disabled => {
                            if value.truthy() {
                                self.dom.set_attr(node, "disabled", "true")?;
                            } else {
                                self.dom.remove_attr(node, "disabled")?;
                            }
                        }
                        DomProp::ClassName => {
                            self.dom.set_attr(node, "class", &value.as_string())?
                        }
                        DomProp::Id => self.dom.set_attr(node, "id", &value.as_string())?,
                        DomProp::Name => self.dom.set_attr(node, "name", &value.as_string())?,
                        DomProp::OffsetWidth
                        | DomProp::OffsetHeight
                        | DomProp::OffsetLeft
                        | DomProp::OffsetTop
                        | DomProp::ScrollWidth
                        | DomProp::ScrollHeight
                        | DomProp::ScrollLeft
                        | DomProp::ScrollTop
                        | DomProp::ActiveElement => {
                            let call = self.describe_dom_prop(prop);
                            return Err(Error::ScriptRuntime(format!("{call} is read-only")));
                        }
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
                    class_names,
                    force,
                } => {
                    let node = self.resolve_dom_query_required_runtime(target, env)?;
                    match method {
                        ClassListMethod::Add => {
                            for class_name in class_names {
                                self.dom.class_add(node, class_name)?;
                            }
                        }
                        ClassListMethod::Remove => {
                            for class_name in class_names {
                                self.dom.class_remove(node, class_name)?;
                            }
                        }
                        ClassListMethod::Toggle => {
                            let class_name = class_names.first().ok_or_else(|| {
                                Error::ScriptRuntime("toggle requires a class name".into())
                            })?;
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
                Stmt::ClassListForEach {
                    target,
                    item_var,
                    index_var,
                    body,
                } => {
                    let node = self.resolve_dom_query_required_runtime(target, env)?;
                    let classes = class_tokens(self.dom.attr(node, "class").as_deref());
                    let prev_item = env.get(item_var).cloned();
                    let prev_index = index_var.as_ref().and_then(|v| env.get(v).cloned());

                    for (idx, class_name) in classes.iter().enumerate() {
                        env.insert(item_var.clone(), Value::String(class_name.clone()));
                        if let Some(index_var) = index_var {
                            env.insert(index_var.clone(), Value::Number(idx as i64));
                        }
                        match self.execute_stmts(body, event_param, event, env)? {
                            ExecFlow::Continue => {}
                            ExecFlow::Break => break,
                            ExecFlow::ContinueLoop => continue,
                            ExecFlow::Return => return Ok(ExecFlow::Return),
                        }
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
                Stmt::NodeTreeMutation {
                    target,
                    method,
                    child,
                    reference,
                } => {
                    let target_node = self.resolve_dom_query_required_runtime(target, env)?;
                    let child = self.eval_expr(child, env, event_param, event)?;
                    let Value::Node(child) = child else {
                        return Err(Error::ScriptRuntime(
                            "before/after/replaceWith/append/appendChild/prepend/removeChild/insertBefore argument must be an element reference".into(),
                        ));
                    };
                    match method {
                        NodeTreeMethod::After => self.dom.insert_after(target_node, child)?,
                        NodeTreeMethod::Append => self.dom.append_child(target_node, child)?,
                        NodeTreeMethod::AppendChild => self.dom.append_child(target_node, child)?,
                        NodeTreeMethod::Before => {
                            let Some(parent) = self.dom.parent(target_node) else {
                                continue;
                            };
                            self.dom.insert_before(parent, child, target_node)?;
                        }
                        NodeTreeMethod::ReplaceWith => {
                            self.dom.replace_with(target_node, child)?;
                        }
                        NodeTreeMethod::Prepend => self.dom.prepend_child(target_node, child)?,
                        NodeTreeMethod::RemoveChild => self.dom.remove_child(target_node, child)?,
                        NodeTreeMethod::InsertBefore => {
                            let Some(reference) = reference else {
                                return Err(Error::ScriptRuntime(
                                    "insertBefore requires reference node".into(),
                                ));
                            };
                            let reference = self.eval_expr(reference, env, event_param, event)?;
                            let Value::Node(reference) = reference else {
                                return Err(Error::ScriptRuntime(
                                    "insertBefore reference must be an element reference".into(),
                                ));
                            };
                            self.dom.insert_before(target_node, child, reference)?;
                        }
                    }
                }
                Stmt::InsertAdjacentElement {
                    target,
                    position,
                    node,
                } => {
                    let target_node = self.resolve_dom_query_required_runtime(target, env)?;
                    let node = self.eval_expr(node, env, event_param, event)?;
                    let Value::Node(node) = node else {
                        return Err(Error::ScriptRuntime(
                            "insertAdjacentElement second argument must be an element reference"
                                .into(),
                        ));
                    };
                    self.dom
                        .insert_adjacent_node(target_node, *position, node)?;
                }
                Stmt::InsertAdjacentText {
                    target,
                    position,
                    text,
                } => {
                    let target_node = self.resolve_dom_query_required_runtime(target, env)?;
                    let text = self.eval_expr(text, env, event_param, event)?;
                    if matches!(
                        position,
                        InsertAdjacentPosition::BeforeBegin | InsertAdjacentPosition::AfterEnd
                    ) && self.dom.parent(target_node).is_none()
                    {
                        continue;
                    }
                    let text_node = self.dom.create_detached_text(text.as_string());
                    self.dom
                        .insert_adjacent_node(target_node, *position, text_node)?;
                }
                Stmt::InsertAdjacentHTML {
                    target,
                    position,
                    html,
                } => {
                    let target_node = self.resolve_dom_query_required_runtime(target, env)?;
                    let position = self.eval_expr(position, env, event_param, event)?;
                    let position = resolve_insert_adjacent_position(&position.as_string())?;
                    let html = self.eval_expr(html, env, event_param, event)?;
                    self.dom
                        .insert_adjacent_html(target_node, position, &html.as_string())?;
                }
                Stmt::SetTimeout { handler, delay_ms } => {
                    let delay = self.eval_expr(delay_ms, env, event_param, event)?;
                    let delay = Self::value_to_i64(&delay);
                    let callback_args = handler
                        .args
                        .iter()
                        .map(|arg| self.eval_expr(arg, env, event_param, event))
                        .collect::<Result<Vec<_>>>()?;
                    let _ =
                        self.schedule_timeout(handler.callback.clone(), delay, callback_args, env);
                }
                Stmt::SetInterval { handler, delay_ms } => {
                    let interval = self.eval_expr(delay_ms, env, event_param, event)?;
                    let interval = Self::value_to_i64(&interval);
                    let callback_args = handler
                        .args
                        .iter()
                        .map(|arg| self.eval_expr(arg, env, event_param, event))
                        .collect::<Result<Vec<_>>>()?;
                    let _ = self.schedule_interval(
                        handler.callback.clone(),
                        interval,
                        callback_args,
                        env,
                    );
                }
                Stmt::QueueMicrotask { handler } => {
                    self.queue_microtask(handler.clone(), env);
                }
                Stmt::ClearTimeout { timer_id } => {
                    let timer_id = self.eval_expr(timer_id, env, event_param, event)?;
                    let timer_id = Self::value_to_i64(&timer_id);
                    self.clear_timeout(timer_id);
                }
                Stmt::NodeRemove { target } => {
                    let node = self.resolve_dom_query_required_runtime(target, env)?;
                    if let Some(active) = self.active_element {
                        if active == node || self.dom.is_descendant_of(active, node) {
                            self.active_element = None;
                            self.dom.set_active_element(None);
                        }
                    }
                    if let Some(active_pseudo) = self.dom.active_pseudo_element() {
                        if active_pseudo == node || self.dom.is_descendant_of(active_pseudo, node) {
                            self.dom.set_active_pseudo_element(None);
                        }
                    }
                    self.dom.remove_node(node)?;
                }
                Stmt::ForEach {
                    target,
                    selector,
                    item_var,
                    index_var,
                    body,
                } => {
                    let items = if let Some(target) = target {
                        match self.resolve_dom_query_runtime(target, env)? {
                            Some(target_node) => {
                                self.dom.query_selector_all_from(&target_node, selector)?
                            }
                            None => Vec::new(),
                        }
                    } else {
                        self.dom.query_selector_all(selector)?
                    };
                    let prev_item = env.get(item_var).cloned();
                    let prev_index = index_var.as_ref().and_then(|v| env.get(v).cloned());

                    for (idx, node) in items.iter().enumerate() {
                        env.insert(item_var.clone(), Value::Node(*node));
                        if let Some(index_var) = index_var {
                            env.insert(index_var.clone(), Value::Number(idx as i64));
                        }
                        match self.execute_stmts(body, event_param, event, env)? {
                            ExecFlow::Continue => {}
                            ExecFlow::Break => break,
                            ExecFlow::ContinueLoop => continue,
                            ExecFlow::Return => return Ok(ExecFlow::Return),
                        }
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
                Stmt::ArrayForEach { target, callback } => {
                    let values = self.resolve_array_from_env(env, target)?;
                    let input = values.borrow().clone();
                    for (idx, item) in input.into_iter().enumerate() {
                        self.execute_array_callback_in_env(
                            callback,
                            &[
                                item,
                                Value::Number(idx as i64),
                                Value::Array(values.clone()),
                            ],
                            env,
                            event,
                        )?;
                    }
                }
                Stmt::For {
                    init,
                    cond,
                    post,
                    body,
                } => {
                    if let Some(init) = init.as_deref() {
                        match self
                            .execute_stmts(std::slice::from_ref(init), event_param, event, env)?
                        {
                            ExecFlow::Continue => {}
                            ExecFlow::Return => return Ok(ExecFlow::Return),
                            ExecFlow::Break => {
                                return Err(Error::ScriptRuntime(
                                    "break statement outside of loop".into(),
                                ))
                            }
                            ExecFlow::ContinueLoop => {
                                return Err(Error::ScriptRuntime(
                                    "continue statement outside of loop".into(),
                                ))
                            }
                        }
                    }

                    loop {
                        let should_run = if let Some(cond) = cond {
                            self.eval_expr(cond, env, event_param, event)?.truthy()
                        } else {
                            true
                        };
                        if !should_run {
                            break;
                        }

                        match self.execute_stmts(body, event_param, event, env)? {
                            ExecFlow::Continue => {}
                            ExecFlow::ContinueLoop => {
                                if let Some(post) = post.as_deref() {
                                    match self.execute_stmts(
                                        std::slice::from_ref(post),
                                        event_param,
                                        event,
                                        env,
                                    )? {
                                        ExecFlow::Continue => {}
                                        ExecFlow::Return => return Ok(ExecFlow::Return),
                                        ExecFlow::Break | ExecFlow::ContinueLoop => {
                                            return Err(Error::ScriptRuntime(
                                                "invalid loop control in post expression".into(),
                                            ));
                                        }
                                    }
                                }
                                continue;
                            }
                            ExecFlow::Break => break,
                            ExecFlow::Return => return Ok(ExecFlow::Return),
                        }
                        if let Some(post) = post.as_deref() {
                            match self.execute_stmts(std::slice::from_ref(post), event_param, event, env)? {
                                ExecFlow::Continue => {}
                                ExecFlow::Return => return Ok(ExecFlow::Return),
                                ExecFlow::Break | ExecFlow::ContinueLoop => {
                                    return Err(Error::ScriptRuntime(
                                        "invalid loop control in post expression".into(),
                                    ))
                                }
                            }
                        }
                    }
                }
                Stmt::ForIn {
                    item_var,
                    iterable,
                    body,
                } => {
                    let iterable = self.eval_expr(iterable, env, event_param, event)?;
                    let items = match iterable {
                        Value::NodeList(nodes) => (0..nodes.len()).collect::<Vec<_>>(),
                        Value::Array(values) => {
                            let values = values.borrow();
                            (0..values.len()).collect::<Vec<_>>()
                        }
                        Value::Null | Value::Undefined => Vec::new(),
                        _ => {
                            return Err(Error::ScriptRuntime(
                                "for...in iterable must be a NodeList or Array".into(),
                            ));
                        }
                    };

                    let prev_item = env.get(item_var).cloned();
                    for idx in items {
                        env.insert(item_var.clone(), Value::Number(idx as i64));
                        match self.execute_stmts(body, event_param, event, env)? {
                            ExecFlow::Continue => {}
                            ExecFlow::ContinueLoop => continue,
                            ExecFlow::Break => break,
                            ExecFlow::Return => return Ok(ExecFlow::Return),
                        }
                    }
                    if let Some(prev) = prev_item {
                        env.insert(item_var.clone(), prev);
                    } else {
                        env.remove(item_var);
                    }
                }
                Stmt::ForOf {
                    item_var,
                    iterable,
                    body,
                } => {
                    let iterable = self.eval_expr(iterable, env, event_param, event)?;
                    let nodes = match iterable {
                        Value::NodeList(nodes) => nodes
                            .into_iter()
                            .map(Value::Node)
                            .collect::<Vec<_>>(),
                        Value::Array(values) => values.borrow().clone(),
                        Value::Null | Value::Undefined => Vec::new(),
                        _ => {
                            return Err(Error::ScriptRuntime(
                                "for...of iterable must be a NodeList or Array".into(),
                            ));
                        }
                    };

                    let prev_item = env.get(item_var).cloned();
                    for item in nodes {
                        env.insert(item_var.clone(), item);
                        match self.execute_stmts(body, event_param, event, env)? {
                            ExecFlow::Continue => {}
                            ExecFlow::ContinueLoop => continue,
                            ExecFlow::Break => break,
                            ExecFlow::Return => return Ok(ExecFlow::Return),
                        }
                    }
                    if let Some(prev) = prev_item {
                        env.insert(item_var.clone(), prev);
                    } else {
                        env.remove(item_var);
                    }
                }
                Stmt::While { cond, body } => {
                    while self.eval_expr(cond, env, event_param, event)?.truthy() {
                        match self.execute_stmts(body, event_param, event, env)? {
                            ExecFlow::Continue => {}
                            ExecFlow::ContinueLoop => continue,
                            ExecFlow::Break => break,
                            ExecFlow::Return => return Ok(ExecFlow::Return),
                        }
                    }
                }
                Stmt::DoWhile { cond, body } => {
                    loop {
                        match self.execute_stmts(body, event_param, event, env)? {
                            ExecFlow::Continue => {}
                            ExecFlow::ContinueLoop => {}
                            ExecFlow::Break => break,
                            ExecFlow::Return => return Ok(ExecFlow::Return),
                        }
                        if !self.eval_expr(cond, env, event_param, event)?.truthy() {
                            break;
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
                        match self.execute_stmts(then_stmts, event_param, event, env)? {
                            ExecFlow::Continue => {}
                            flow => return Ok(flow),
                        }
                    } else {
                        match self.execute_stmts(else_stmts, event_param, event, env)? {
                            ExecFlow::Continue => {}
                            flow => return Ok(flow),
                        }
                    }
                }
                Stmt::Return { value } => {
                    let return_value = if let Some(value) = value {
                        self.eval_expr(value, env, event_param, event)?
                    } else {
                        Value::Undefined
                    };
                    env.insert(INTERNAL_RETURN_SLOT.to_string(), return_value);
                    return Ok(ExecFlow::Return);
                }
                Stmt::Break => {
                    return Ok(ExecFlow::Break);
                }
                Stmt::Continue => {
                    return Ok(ExecFlow::ContinueLoop);
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
                Stmt::DomMethodCall { target, method } => {
                    let node = self.resolve_dom_query_required_runtime(target, env)?;
                    match method {
                        DomMethod::Focus => self.focus_node_with_env(node, env)?,
                        DomMethod::Blur => self.blur_node_with_env(node, env)?,
                        DomMethod::Click => self.click_node_with_env(node, env)?,
                        DomMethod::Submit => self.submit_form_with_env(node, env)?,
                        DomMethod::Reset => self.reset_form_with_env(node, env)?,
                        DomMethod::ScrollIntoView => self.scroll_into_view_node_with_env(node, env)?,
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
                    let _ = self.dispatch_event_with_env(node, &event_name, env, false)?;
                }
                Stmt::Expr(expr) => {
                    let _ = self.eval_expr(expr, env, event_param, event)?;
                }
            }
        }

        Ok(ExecFlow::Continue)
    }

    fn bind_timer_id_to_task_env(&mut self, name: &str, expr: &Expr, value: &Value) {
        if !matches!(expr, Expr::SetTimeout { .. } | Expr::SetInterval { .. }) {
            return;
        }
        let Value::Number(timer_id) = value else {
            return;
        };
        for task in self
            .task_queue
            .iter_mut()
            .filter(|task| task.id == *timer_id)
        {
            task.env.insert(name.to_string(), value.clone());
        }
    }

    fn eval_expr(
        &mut self,
        expr: &Expr,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        match expr {
            Expr::String(value) => Ok(Value::String(value.clone())),
            Expr::Bool(value) => Ok(Value::Bool(*value)),
            Expr::Null => Ok(Value::Null),
            Expr::Undefined => Ok(Value::Undefined),
            Expr::Number(value) => Ok(Value::Number(*value)),
            Expr::Float(value) => Ok(Value::Float(*value)),
            Expr::DateNow => Ok(Value::Number(self.now_ms)),
            Expr::DateNew { value } => {
                let timestamp_ms = if let Some(value) = value {
                    let value = self.eval_expr(value, env, event_param, event)?;
                    self.coerce_date_timestamp_ms(&value)
                } else {
                    self.now_ms
                };
                Ok(Self::new_date_value(timestamp_ms))
            }
            Expr::DateParse(value) => {
                let value = self.eval_expr(value, env, event_param, event)?.as_string();
                if let Some(timestamp_ms) = Self::parse_date_string_to_epoch_ms(&value) {
                    Ok(Value::Number(timestamp_ms))
                } else {
                    Ok(Value::Float(f64::NAN))
                }
            }
            Expr::DateUtc { args } => {
                let mut values = Vec::with_capacity(args.len());
                for arg in args {
                    let value = self.eval_expr(arg, env, event_param, event)?;
                    values.push(Self::value_to_i64(&value));
                }

                let mut year = values.first().copied().unwrap_or(0);
                if (0..=99).contains(&year) {
                    year += 1900;
                }
                let month = values.get(1).copied().unwrap_or(0);
                let day = values.get(2).copied().unwrap_or(1);
                let hour = values.get(3).copied().unwrap_or(0);
                let minute = values.get(4).copied().unwrap_or(0);
                let second = values.get(5).copied().unwrap_or(0);

                Ok(Value::Number(Self::utc_timestamp_ms_from_components(
                    year, month, day, hour, minute, second, 0,
                )))
            }
            Expr::DateGetTime(target) => {
                let date = self.resolve_date_from_env(env, target)?;
                Ok(Value::Number(*date.borrow()))
            }
            Expr::DateSetTime { target, value } => {
                let date = self.resolve_date_from_env(env, target)?;
                let value = self.eval_expr(value, env, event_param, event)?;
                let timestamp_ms = Self::value_to_i64(&value);
                *date.borrow_mut() = timestamp_ms;
                Ok(Value::Number(timestamp_ms))
            }
            Expr::DateToIsoString(target) => {
                let date = self.resolve_date_from_env(env, target)?;
                Ok(Value::String(Self::format_iso_8601_utc(*date.borrow())))
            }
            Expr::DateGetFullYear(target) => {
                let date = self.resolve_date_from_env(env, target)?;
                let (year, ..) = Self::date_components_utc(*date.borrow());
                Ok(Value::Number(year))
            }
            Expr::DateGetMonth(target) => {
                let date = self.resolve_date_from_env(env, target)?;
                let (_, month, ..) = Self::date_components_utc(*date.borrow());
                Ok(Value::Number((month as i64) - 1))
            }
            Expr::DateGetDate(target) => {
                let date = self.resolve_date_from_env(env, target)?;
                let (_, _, day, ..) = Self::date_components_utc(*date.borrow());
                Ok(Value::Number(day as i64))
            }
            Expr::DateGetHours(target) => {
                let date = self.resolve_date_from_env(env, target)?;
                let (_, _, _, hour, ..) = Self::date_components_utc(*date.borrow());
                Ok(Value::Number(hour as i64))
            }
            Expr::DateGetMinutes(target) => {
                let date = self.resolve_date_from_env(env, target)?;
                let (_, _, _, _, minute, ..) = Self::date_components_utc(*date.borrow());
                Ok(Value::Number(minute as i64))
            }
            Expr::DateGetSeconds(target) => {
                let date = self.resolve_date_from_env(env, target)?;
                let (_, _, _, _, _, second, _) = Self::date_components_utc(*date.borrow());
                Ok(Value::Number(second as i64))
            }
            Expr::MathRandom => Ok(Value::Float(self.next_random_f64())),
            Expr::EncodeUri(value) => {
                let value = self.eval_expr(value, env, event_param, event)?;
                Ok(Value::String(encode_uri_like(&value.as_string(), false)))
            }
            Expr::EncodeUriComponent(value) => {
                let value = self.eval_expr(value, env, event_param, event)?;
                Ok(Value::String(encode_uri_like(&value.as_string(), true)))
            }
            Expr::DecodeUri(value) => {
                let value = self.eval_expr(value, env, event_param, event)?;
                Ok(Value::String(decode_uri_like(&value.as_string(), false)?))
            }
            Expr::DecodeUriComponent(value) => {
                let value = self.eval_expr(value, env, event_param, event)?;
                Ok(Value::String(decode_uri_like(&value.as_string(), true)?))
            }
            Expr::Escape(value) => {
                let value = self.eval_expr(value, env, event_param, event)?;
                Ok(Value::String(js_escape(&value.as_string())))
            }
            Expr::Unescape(value) => {
                let value = self.eval_expr(value, env, event_param, event)?;
                Ok(Value::String(js_unescape(&value.as_string())))
            }
            Expr::IsNaN(value) => {
                let value = self.eval_expr(value, env, event_param, event)?;
                Ok(Value::Bool(Self::coerce_number_for_global(&value).is_nan()))
            }
            Expr::IsFinite(value) => {
                let value = self.eval_expr(value, env, event_param, event)?;
                Ok(Value::Bool(Self::coerce_number_for_global(&value).is_finite()))
            }
            Expr::Atob(value) => {
                let value = self.eval_expr(value, env, event_param, event)?;
                Ok(Value::String(decode_base64_to_binary_string(
                    &value.as_string(),
                )?))
            }
            Expr::Btoa(value) => {
                let value = self.eval_expr(value, env, event_param, event)?;
                Ok(Value::String(encode_binary_string_to_base64(
                    &value.as_string(),
                )?))
            }
            Expr::ParseInt { value, radix } => {
                let value = self.eval_expr(value, env, event_param, event)?;
                let radix = radix
                    .as_ref()
                    .map(|expr| self.eval_expr(expr, env, event_param, event))
                    .transpose()?
                    .map(|radix| Self::value_to_i64(&radix));
                Ok(Value::Float(parse_js_parse_int(&value.as_string(), radix)))
            }
            Expr::ParseFloat(value) => {
                let value = self.eval_expr(value, env, event_param, event)?;
                Ok(Value::Float(parse_js_parse_float(&value.as_string())))
            }
            Expr::ArrayLiteral(values) => {
                let mut out = Vec::with_capacity(values.len());
                for value in values {
                    out.push(self.eval_expr(value, env, event_param, event)?);
                }
                Ok(Self::new_array_value(out))
            }
            Expr::ArrayIsArray(value) => {
                let value = self.eval_expr(value, env, event_param, event)?;
                Ok(Value::Bool(matches!(value, Value::Array(_))))
            }
            Expr::ArrayLength(target) => {
                match env.get(target) {
                    Some(Value::Array(values)) => Ok(Value::Number(values.borrow().len() as i64)),
                    Some(Value::NodeList(nodes)) => Ok(Value::Number(nodes.len() as i64)),
                    Some(Value::String(value)) => Ok(Value::Number(value.chars().count() as i64)),
                    Some(_) => Err(Error::ScriptRuntime(format!(
                        "variable '{}' is not an array",
                        target
                    ))),
                    None => Err(Error::ScriptRuntime(format!("unknown variable: {}", target))),
                }
            }
            Expr::ArrayIndex { target, index } => {
                let index = self.eval_expr(index, env, event_param, event)?;
                let Some(index) = self.value_as_index(&index) else {
                    return Ok(Value::Undefined);
                };
                match env.get(target) {
                    Some(Value::Array(values)) => Ok(values
                        .borrow()
                        .get(index)
                        .cloned()
                        .unwrap_or(Value::Undefined)),
                    Some(Value::NodeList(nodes)) => Ok(nodes
                        .get(index)
                        .copied()
                        .map(Value::Node)
                        .unwrap_or(Value::Undefined)),
                    Some(Value::String(value)) => Ok(value
                        .chars()
                        .nth(index)
                        .map(|ch| Value::String(ch.to_string()))
                        .unwrap_or(Value::Undefined)),
                    Some(_) => Err(Error::ScriptRuntime(format!(
                        "variable '{}' is not an array",
                        target
                    ))),
                    None => Err(Error::ScriptRuntime(format!("unknown variable: {}", target))),
                }
            }
            Expr::ArrayPush { target, args } => {
                let values = self.resolve_array_from_env(env, target)?;
                let mut evaluated = Vec::with_capacity(args.len());
                for arg in args {
                    evaluated.push(self.eval_expr(arg, env, event_param, event)?);
                }
                let mut values = values.borrow_mut();
                values.extend(evaluated);
                Ok(Value::Number(values.len() as i64))
            }
            Expr::ArrayPop(target) => {
                let values = self.resolve_array_from_env(env, target)?;
                Ok(values.borrow_mut().pop().unwrap_or(Value::Undefined))
            }
            Expr::ArrayShift(target) => {
                let values = self.resolve_array_from_env(env, target)?;
                let mut values = values.borrow_mut();
                if values.is_empty() {
                    Ok(Value::Undefined)
                } else {
                    Ok(values.remove(0))
                }
            }
            Expr::ArrayUnshift { target, args } => {
                let values = self.resolve_array_from_env(env, target)?;
                let mut evaluated = Vec::with_capacity(args.len());
                for arg in args {
                    evaluated.push(self.eval_expr(arg, env, event_param, event)?);
                }
                let mut values = values.borrow_mut();
                for value in evaluated.into_iter().rev() {
                    values.insert(0, value);
                }
                Ok(Value::Number(values.len() as i64))
            }
            Expr::ArrayMap { target, callback } => {
                let values = self.resolve_array_from_env(env, target)?;
                let input = values.borrow().clone();
                let mut out = Vec::with_capacity(input.len());
                for (idx, item) in input.into_iter().enumerate() {
                    let mapped = self.execute_array_callback(
                        callback,
                        &[
                            item,
                            Value::Number(idx as i64),
                            Value::Array(values.clone()),
                        ],
                        env,
                        event,
                    )?;
                    out.push(mapped);
                }
                Ok(Self::new_array_value(out))
            }
            Expr::ArrayFilter { target, callback } => {
                let values = self.resolve_array_from_env(env, target)?;
                let input = values.borrow().clone();
                let mut out = Vec::new();
                for (idx, item) in input.into_iter().enumerate() {
                    let keep = self.execute_array_callback(
                        callback,
                        &[
                            item.clone(),
                            Value::Number(idx as i64),
                            Value::Array(values.clone()),
                        ],
                        env,
                        event,
                    )?;
                    if keep.truthy() {
                        out.push(item);
                    }
                }
                Ok(Self::new_array_value(out))
            }
            Expr::ArrayReduce {
                target,
                callback,
                initial,
            } => {
                let values = self.resolve_array_from_env(env, target)?;
                let input = values.borrow().clone();
                let mut start_index = 0usize;
                let mut acc = if let Some(initial) = initial {
                    self.eval_expr(initial, env, event_param, event)?
                } else {
                    let Some(first) = input.first().cloned() else {
                        return Err(Error::ScriptRuntime(
                            "reduce of empty array with no initial value".into(),
                        ));
                    };
                    start_index = 1;
                    first
                };
                for (idx, item) in input.into_iter().enumerate().skip(start_index) {
                    acc = self.execute_array_callback(
                        callback,
                        &[
                            acc,
                            item,
                            Value::Number(idx as i64),
                            Value::Array(values.clone()),
                        ],
                        env,
                        event,
                    )?;
                }
                Ok(acc)
            }
            Expr::ArrayForEach { target, callback } => {
                let values = self.resolve_array_from_env(env, target)?;
                let input = values.borrow().clone();
                for (idx, item) in input.into_iter().enumerate() {
                    let _ = self.execute_array_callback(
                        callback,
                        &[
                            item,
                            Value::Number(idx as i64),
                            Value::Array(values.clone()),
                        ],
                        env,
                        event,
                    )?;
                }
                Ok(Value::Undefined)
            }
            Expr::ArrayFind { target, callback } => {
                let values = self.resolve_array_from_env(env, target)?;
                let input = values.borrow().clone();
                for (idx, item) in input.into_iter().enumerate() {
                    let matched = self.execute_array_callback(
                        callback,
                        &[
                            item.clone(),
                            Value::Number(idx as i64),
                            Value::Array(values.clone()),
                        ],
                        env,
                        event,
                    )?;
                    if matched.truthy() {
                        return Ok(item);
                    }
                }
                Ok(Value::Undefined)
            }
            Expr::ArraySome { target, callback } => {
                let values = self.resolve_array_from_env(env, target)?;
                let input = values.borrow().clone();
                for (idx, item) in input.into_iter().enumerate() {
                    let matched = self.execute_array_callback(
                        callback,
                        &[
                            item,
                            Value::Number(idx as i64),
                            Value::Array(values.clone()),
                        ],
                        env,
                        event,
                    )?;
                    if matched.truthy() {
                        return Ok(Value::Bool(true));
                    }
                }
                Ok(Value::Bool(false))
            }
            Expr::ArrayEvery { target, callback } => {
                let values = self.resolve_array_from_env(env, target)?;
                let input = values.borrow().clone();
                for (idx, item) in input.into_iter().enumerate() {
                    let matched = self.execute_array_callback(
                        callback,
                        &[
                            item,
                            Value::Number(idx as i64),
                            Value::Array(values.clone()),
                        ],
                        env,
                        event,
                    )?;
                    if !matched.truthy() {
                        return Ok(Value::Bool(false));
                    }
                }
                Ok(Value::Bool(true))
            }
            Expr::ArrayIncludes {
                target,
                search,
                from_index,
            } => {
                let search = self.eval_expr(search, env, event_param, event)?;
                match env.get(target) {
                    Some(Value::Array(values)) => {
                        let values = values.borrow();
                        let len = values.len() as i64;
                        let mut start = from_index
                            .as_ref()
                            .map(|value| self.eval_expr(value, env, event_param, event))
                            .transpose()?
                            .map(|value| Self::value_to_i64(&value))
                            .unwrap_or(0);
                        if start < 0 {
                            start = (len + start).max(0);
                        }
                        let start = start.min(len) as usize;
                        for value in values.iter().skip(start) {
                            if self.strict_equal(value, &search) {
                                return Ok(Value::Bool(true));
                            }
                        }
                        Ok(Value::Bool(false))
                    }
                    Some(Value::String(value)) => {
                        let search = search.as_string();
                        let len = value.chars().count() as i64;
                        let mut start = from_index
                            .as_ref()
                            .map(|value| self.eval_expr(value, env, event_param, event))
                            .transpose()?
                            .map(|value| Self::value_to_i64(&value))
                            .unwrap_or(0);
                        if start < 0 {
                            start = (len + start).max(0);
                        }
                        let start = start.min(len) as usize;
                        let start_byte = Self::char_index_to_byte(value, start);
                        Ok(Value::Bool(value[start_byte..].contains(&search)))
                    }
                    Some(_) => Err(Error::ScriptRuntime(format!(
                        "variable '{}' is not an array",
                        target
                    ))),
                    None => Err(Error::ScriptRuntime(format!("unknown variable: {}", target))),
                }
            }
            Expr::ArraySlice { target, start, end } => {
                match env.get(target) {
                    Some(Value::Array(values)) => {
                        let values = values.borrow();
                        let len = values.len();
                        let start = start
                            .as_ref()
                            .map(|value| self.eval_expr(value, env, event_param, event))
                            .transpose()?
                            .map(|value| Self::value_to_i64(&value))
                            .map(|value| Self::normalize_slice_index(len, value))
                            .unwrap_or(0);
                        let end = end
                            .as_ref()
                            .map(|value| self.eval_expr(value, env, event_param, event))
                            .transpose()?
                            .map(|value| Self::value_to_i64(&value))
                            .map(|value| Self::normalize_slice_index(len, value))
                            .unwrap_or(len);
                        let end = end.max(start);
                        Ok(Self::new_array_value(values[start..end].to_vec()))
                    }
                    Some(Value::String(value)) => {
                        let len = value.chars().count();
                        let start = start
                            .as_ref()
                            .map(|value| self.eval_expr(value, env, event_param, event))
                            .transpose()?
                            .map(|value| Self::value_to_i64(&value))
                            .map(|value| Self::normalize_slice_index(len, value))
                            .unwrap_or(0);
                        let end = end
                            .as_ref()
                            .map(|value| self.eval_expr(value, env, event_param, event))
                            .transpose()?
                            .map(|value| Self::value_to_i64(&value))
                            .map(|value| Self::normalize_slice_index(len, value))
                            .unwrap_or(len);
                        let end = end.max(start);
                        Ok(Value::String(Self::substring_chars(value, start, end)))
                    }
                    Some(_) => Err(Error::ScriptRuntime(format!(
                        "variable '{}' is not an array",
                        target
                    ))),
                    None => Err(Error::ScriptRuntime(format!("unknown variable: {}", target))),
                }
            }
            Expr::ArraySplice {
                target,
                start,
                delete_count,
                items,
            } => {
                let values = self.resolve_array_from_env(env, target)?;
                let start = self.eval_expr(start, env, event_param, event)?;
                let start = Self::value_to_i64(&start);
                let delete_count = delete_count
                    .as_ref()
                    .map(|value| self.eval_expr(value, env, event_param, event))
                    .transpose()?
                    .map(|value| Self::value_to_i64(&value));
                let mut insert_items = Vec::with_capacity(items.len());
                for item in items {
                    insert_items.push(self.eval_expr(item, env, event_param, event)?);
                }

                let mut values = values.borrow_mut();
                let len = values.len();
                let start = Self::normalize_splice_start_index(len, start);
                let delete_count = delete_count
                    .unwrap_or((len.saturating_sub(start)) as i64)
                    .max(0) as usize;
                let delete_count = delete_count.min(len.saturating_sub(start));
                let removed = values
                    .drain(start..start + delete_count)
                    .collect::<Vec<_>>();
                for (offset, item) in insert_items.into_iter().enumerate() {
                    values.insert(start + offset, item);
                }
                Ok(Self::new_array_value(removed))
            }
            Expr::ArrayJoin { target, separator } => {
                let values = self.resolve_array_from_env(env, target)?;
                let separator = separator
                    .as_ref()
                    .map(|value| self.eval_expr(value, env, event_param, event))
                    .transpose()?
                    .map(|value| value.as_string())
                    .unwrap_or_else(|| ",".to_string());
                let values = values.borrow();
                let mut out = String::new();
                for (idx, value) in values.iter().enumerate() {
                    if idx > 0 {
                        out.push_str(&separator);
                    }
                    if matches!(value, Value::Null | Value::Undefined) {
                        continue;
                    }
                    out.push_str(&value.as_string());
                }
                Ok(Value::String(out))
            }
            Expr::StringTrim { value, mode } => {
                let value = self.eval_expr(value, env, event_param, event)?.as_string();
                let value = match mode {
                    StringTrimMode::Both => value.trim().to_string(),
                    StringTrimMode::Start => value.trim_start().to_string(),
                    StringTrimMode::End => value.trim_end().to_string(),
                };
                Ok(Value::String(value))
            }
            Expr::StringToUpperCase(value) => {
                let value = self.eval_expr(value, env, event_param, event)?.as_string();
                Ok(Value::String(value.to_uppercase()))
            }
            Expr::StringToLowerCase(value) => {
                let value = self.eval_expr(value, env, event_param, event)?.as_string();
                Ok(Value::String(value.to_lowercase()))
            }
            Expr::StringIncludes {
                value,
                search,
                position,
            } => {
                let value = self.eval_expr(value, env, event_param, event)?.as_string();
                let search = self.eval_expr(search, env, event_param, event)?.as_string();
                let len = value.chars().count() as i64;
                let mut position = position
                    .as_ref()
                    .map(|value| self.eval_expr(value, env, event_param, event))
                    .transpose()?
                    .map(|value| Self::value_to_i64(&value))
                    .unwrap_or(0);
                if position < 0 {
                    position = 0;
                }
                let position = position.min(len) as usize;
                let position_byte = Self::char_index_to_byte(&value, position);
                Ok(Value::Bool(value[position_byte..].contains(&search)))
            }
            Expr::StringStartsWith {
                value,
                search,
                position,
            } => {
                let value = self.eval_expr(value, env, event_param, event)?.as_string();
                let search = self.eval_expr(search, env, event_param, event)?.as_string();
                let len = value.chars().count() as i64;
                let mut position = position
                    .as_ref()
                    .map(|value| self.eval_expr(value, env, event_param, event))
                    .transpose()?
                    .map(|value| Self::value_to_i64(&value))
                    .unwrap_or(0);
                if position < 0 {
                    position = 0;
                }
                let position = position.min(len) as usize;
                let position_byte = Self::char_index_to_byte(&value, position);
                Ok(Value::Bool(value[position_byte..].starts_with(&search)))
            }
            Expr::StringEndsWith {
                value,
                search,
                length,
            } => {
                let value = self.eval_expr(value, env, event_param, event)?.as_string();
                let search = self.eval_expr(search, env, event_param, event)?.as_string();
                let len = value.chars().count();
                let end = length
                    .as_ref()
                    .map(|value| self.eval_expr(value, env, event_param, event))
                    .transpose()?
                    .map(|value| Self::value_to_i64(&value))
                    .map(|value| {
                        if value < 0 {
                            0
                        } else {
                            (value as usize).min(len)
                        }
                    })
                    .unwrap_or(len);
                let hay = Self::substring_chars(&value, 0, end);
                Ok(Value::Bool(hay.ends_with(&search)))
            }
            Expr::StringSlice { value, start, end } => {
                let value = self.eval_expr(value, env, event_param, event)?.as_string();
                let len = value.chars().count();
                let start = start
                    .as_ref()
                    .map(|value| self.eval_expr(value, env, event_param, event))
                    .transpose()?
                    .map(|value| Self::value_to_i64(&value))
                    .map(|value| Self::normalize_slice_index(len, value))
                    .unwrap_or(0);
                let end = end
                    .as_ref()
                    .map(|value| self.eval_expr(value, env, event_param, event))
                    .transpose()?
                    .map(|value| Self::value_to_i64(&value))
                    .map(|value| Self::normalize_slice_index(len, value))
                    .unwrap_or(len);
                let end = end.max(start);
                Ok(Value::String(Self::substring_chars(&value, start, end)))
            }
            Expr::StringSubstring { value, start, end } => {
                let value = self.eval_expr(value, env, event_param, event)?.as_string();
                let len = value.chars().count();
                let start = start
                    .as_ref()
                    .map(|value| self.eval_expr(value, env, event_param, event))
                    .transpose()?
                    .map(|value| Self::value_to_i64(&value))
                    .map(|value| Self::normalize_substring_index(len, value))
                    .unwrap_or(0);
                let end = end
                    .as_ref()
                    .map(|value| self.eval_expr(value, env, event_param, event))
                    .transpose()?
                    .map(|value| Self::value_to_i64(&value))
                    .map(|value| Self::normalize_substring_index(len, value))
                    .unwrap_or(len);
                let (start, end) = if start <= end {
                    (start, end)
                } else {
                    (end, start)
                };
                Ok(Value::String(Self::substring_chars(&value, start, end)))
            }
            Expr::StringSplit {
                value,
                separator,
                limit,
            } => {
                let value = self.eval_expr(value, env, event_param, event)?.as_string();
                let separator = separator
                    .as_ref()
                    .map(|value| self.eval_expr(value, env, event_param, event))
                    .transpose()?
                    .map(|value| value.as_string());
                let limit = limit
                    .as_ref()
                    .map(|value| self.eval_expr(value, env, event_param, event))
                    .transpose()?
                    .map(|value| Self::value_to_i64(&value));
                Ok(Self::new_array_value(Self::split_string(&value, separator, limit)))
            }
            Expr::StringReplace { value, from, to } => {
                let value = self.eval_expr(value, env, event_param, event)?.as_string();
                let from = self.eval_expr(from, env, event_param, event)?.as_string();
                let to = self.eval_expr(to, env, event_param, event)?.as_string();
                Ok(Value::String(value.replacen(&from, &to, 1)))
            }
            Expr::StringIndexOf {
                value,
                search,
                position,
            } => {
                let value = self.eval_expr(value, env, event_param, event)?.as_string();
                let search = self.eval_expr(search, env, event_param, event)?.as_string();
                let len = value.chars().count() as i64;
                let mut position = position
                    .as_ref()
                    .map(|value| self.eval_expr(value, env, event_param, event))
                    .transpose()?
                    .map(|value| Self::value_to_i64(&value))
                    .unwrap_or(0);
                if position < 0 {
                    position = 0;
                }
                let position = position.min(len) as usize;
                Ok(Value::Number(
                    Self::string_index_of(&value, &search, position)
                        .map(|value| value as i64)
                        .unwrap_or(-1),
                ))
            }
            Expr::Fetch(request) => {
                let request = self.eval_expr(request, env, event_param, event)?.as_string();
                self.fetch_calls.push(request.clone());
                let response = self.fetch_mocks.get(&request).cloned().ok_or_else(|| {
                    Error::ScriptRuntime(format!("fetch mock not found for request: {request}"))
                })?;
                Ok(Value::String(response))
            }
            Expr::Alert(message) => {
                let message = self.eval_expr(message, env, event_param, event)?.as_string();
                self.alert_messages.push(message);
                Ok(Value::Undefined)
            }
            Expr::Confirm(message) => {
                let _ = self.eval_expr(message, env, event_param, event)?;
                let accepted = self
                    .confirm_responses
                    .pop_front()
                    .unwrap_or(self.default_confirm_response);
                Ok(Value::Bool(accepted))
            }
            Expr::Prompt { message, default } => {
                let _ = self.eval_expr(message, env, event_param, event)?;
                let default_value = default
                    .as_ref()
                    .map(|value| self.eval_expr(value, env, event_param, event))
                    .transpose()?
                    .map(|value| value.as_string());
                let response = self
                    .prompt_responses
                    .pop_front()
                    .unwrap_or_else(|| self.default_prompt_response.clone().or(default_value));
                match response {
                    Some(value) => Ok(Value::String(value)),
                    None => Ok(Value::Null),
                }
            }
            Expr::Var(name) => env
                .get(name)
                .cloned()
                .ok_or_else(|| Error::ScriptRuntime(format!("unknown variable: {name}"))),
            Expr::DomRef(target) => {
                let is_list_query = matches!(
                    target,
                    DomQuery::BySelectorAll { .. } | DomQuery::QuerySelectorAll { .. }
                );
                if is_list_query {
                    let nodes = self
                        .resolve_dom_query_list_runtime(target, env)?
                        .unwrap_or_default();
                    Ok(Value::NodeList(nodes))
                } else {
                    let node = self.resolve_dom_query_required_runtime(target, env)?;
                    Ok(Value::Node(node))
                }
            }
            Expr::CreateElement(tag_name) => {
                let node = self.dom.create_detached_element(tag_name.clone());
                Ok(Value::Node(node))
            }
            Expr::CreateTextNode(text) => {
                let node = self.dom.create_detached_text(text.clone());
                Ok(Value::Node(node))
            }
            Expr::Function { handler } => Ok(Value::Function(handler.clone())),
            Expr::SetTimeout { handler, delay_ms } => {
                let delay = self.eval_expr(delay_ms, env, event_param, event)?;
                let delay = Self::value_to_i64(&delay);
                let callback_args = handler
                    .args
                    .iter()
                    .map(|arg| self.eval_expr(arg, env, event_param, event))
                    .collect::<Result<Vec<_>>>()?;
                let id = self.schedule_timeout(handler.callback.clone(), delay, callback_args, env);
                Ok(Value::Number(id))
            }
            Expr::SetInterval { handler, delay_ms } => {
                let interval = self.eval_expr(delay_ms, env, event_param, event)?;
                let interval = Self::value_to_i64(&interval);
                let callback_args = handler
                    .args
                    .iter()
                    .map(|arg| self.eval_expr(arg, env, event_param, event))
                    .collect::<Result<Vec<_>>>()?;
                let id = self.schedule_interval(handler.callback.clone(), interval, callback_args, env);
                Ok(Value::Number(id))
            }
            Expr::QueueMicrotask { handler } => {
                self.queue_microtask(handler.clone(), env);
                Ok(Value::Null)
            }
            Expr::PromiseThen { callback } => {
                self.queue_microtask(callback.clone(), env);
                Ok(Value::Null)
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
                    DomProp::Readonly => Ok(Value::Bool(self.dom.readonly(node))),
                    DomProp::Disabled => Ok(Value::Bool(self.dom.disabled(node))),
                    DomProp::Required => Ok(Value::Bool(self.dom.required(node))),
                    DomProp::TextContent => Ok(Value::String(self.dom.text_content(node))),
                    DomProp::InnerHtml => Ok(Value::String(self.dom.inner_html(node)?)),
                    DomProp::ClassName => Ok(Value::String(
                        self.dom.attr(node, "class").unwrap_or_default(),
                    )),
                    DomProp::Id => Ok(Value::String(self.dom.attr(node, "id").unwrap_or_default())),
                    DomProp::Name => Ok(Value::String(
                        self.dom.attr(node, "name").unwrap_or_default(),
                    )),
                    DomProp::Dataset(key) => Ok(Value::String(self.dom.dataset_get(node, key)?)),
                    DomProp::Style(prop) => Ok(Value::String(self.dom.style_get(node, prop)?)),
                    DomProp::OffsetWidth => Ok(Value::Number(self.dom.offset_width(node)?)),
                    DomProp::OffsetHeight => Ok(Value::Number(self.dom.offset_height(node)?)),
                    DomProp::OffsetLeft => Ok(Value::Number(self.dom.offset_left(node)?)),
                    DomProp::OffsetTop => Ok(Value::Number(self.dom.offset_top(node)?)),
                    DomProp::ScrollWidth => Ok(Value::Number(self.dom.scroll_width(node)?)),
                    DomProp::ScrollHeight => Ok(Value::Number(self.dom.scroll_height(node)?)),
                    DomProp::ScrollLeft => Ok(Value::Number(self.dom.scroll_left(node)?)),
                    DomProp::ScrollTop => Ok(Value::Number(self.dom.scroll_top(node)?)),
                    DomProp::ActiveElement => Ok(self
                        .dom
                        .active_element()
                        .map(Value::Node)
                        .unwrap_or(Value::Null)),
                }
            }
            Expr::DomMatches { target, selector } => {
                let node = self.resolve_dom_query_required_runtime(target, env)?;
                let result = self.dom.matches_selector(node, selector)?;
                Ok(Value::Bool(result))
            }
            Expr::DomClosest { target, selector } => {
                let node = self.resolve_dom_query_required_runtime(target, env)?;
                let result = self.dom.closest(node, selector)?;
                Ok(result.map_or(Value::Null, Value::Node))
            }
            Expr::DomComputedStyleProperty { target, property } => {
                let node = self.resolve_dom_query_required_runtime(target, env)?;
                Ok(Value::String(self.dom.style_get(node, property)?))
            }
            Expr::ClassListContains { target, class_name } => {
                let node = self.resolve_dom_query_required_runtime(target, env)?;
                Ok(Value::Bool(self.dom.class_contains(node, class_name)?))
            }
            Expr::QuerySelectorAllLength { target } => {
                let len = self
                    .resolve_dom_query_list_runtime(target, env)?
                    .unwrap_or_default()
                    .len() as i64;
                Ok(Value::Number(len))
            }
            Expr::FormElementsLength { form } => {
                let form_node = self.resolve_dom_query_required_runtime(form, env)?;
                let len = self.form_elements(form_node)?.len() as i64;
                Ok(Value::Number(len))
            }
            Expr::FormDataNew { form } => {
                let form_node = self.resolve_dom_query_required_runtime(form, env)?;
                Ok(Value::FormData(self.form_data_entries(form_node)?))
            }
            Expr::FormDataGet { source, name } => {
                let entries = self.eval_form_data_source(source, env)?;
                let value = entries
                    .iter()
                    .find_map(|(entry_name, value)| (entry_name == name).then(|| value.clone()))
                    .unwrap_or_default();
                Ok(Value::String(value))
            }
            Expr::FormDataHas { source, name } => {
                let entries = self.eval_form_data_source(source, env)?;
                let has = entries.iter().any(|(entry_name, _)| entry_name == name);
                Ok(Value::Bool(has))
            }
            Expr::FormDataGetAllLength { source, name } => {
                let entries = self.eval_form_data_source(source, env)?;
                let len = entries
                    .iter()
                    .filter(|(entry_name, _)| entry_name == name)
                    .count() as i64;
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
                    EventExprProp::TargetName => {
                        Value::String(self.dom.attr(event.target, "name").unwrap_or_default())
                    }
                    EventExprProp::CurrentTargetName => Value::String(
                        self.dom
                            .attr(event.current_target, "name")
                            .unwrap_or_default(),
                    ),
                    EventExprProp::DefaultPrevented => Value::Bool(event.default_prevented),
                    EventExprProp::IsTrusted => Value::Bool(event.is_trusted),
                    EventExprProp::Bubbles => Value::Bool(event.bubbles),
                    EventExprProp::Cancelable => Value::Bool(event.cancelable),
                    EventExprProp::TargetId => {
                        Value::String(self.dom.attr(event.target, "id").unwrap_or_default())
                    }
                    EventExprProp::CurrentTargetId => Value::String(
                        self.dom
                            .attr(event.current_target, "id")
                            .unwrap_or_default(),
                    ),
                    EventExprProp::EventPhase => Value::Number(event.event_phase as i64),
                    EventExprProp::TimeStamp => Value::Number(event.time_stamp_ms),
                };
                Ok(value)
            }
            Expr::Neg(inner) => {
                let value = self.eval_expr(inner, env, event_param, event)?;
                match value {
                    Value::Number(v) => Ok(Value::Number(-v)),
                    Value::Float(v) => Ok(Value::Float(-v)),
                    other => Ok(Value::Float(-self.numeric_value(&other))),
                }
            }
            Expr::Pos(inner) => {
                let value = self.eval_expr(inner, env, event_param, event)?;
                Ok(Value::Float(self.numeric_value(&value)))
            }
            Expr::BitNot(inner) => {
                let value = self.eval_expr(inner, env, event_param, event)?;
                Ok(Value::Number((!self.to_i32_for_bitwise(&value)) as i64))
            }
            Expr::Not(inner) => {
                let value = self.eval_expr(inner, env, event_param, event)?;
                Ok(Value::Bool(!value.truthy()))
            }
            Expr::Void(inner) => {
                self.eval_expr(inner, env, event_param, event)?;
                Ok(Value::Undefined)
            }
            Expr::Delete(inner) => match inner.as_ref() {
                Expr::Var(name) => Ok(Value::Bool(!env.contains_key(name))),
                _ => {
                    self.eval_expr(inner, env, event_param, event)?;
                    Ok(Value::Bool(true))
                }
            },
                    Expr::TypeOf(inner) => {
                let js_type = match inner.as_ref() {
                    Expr::Var(name) => env.get(name).map_or("undefined", |value| match value {
                        Value::Null => "object",
                        Value::Bool(_) => "boolean",
                        Value::Number(_) | Value::Float(_) => "number",
                        Value::Undefined => "undefined",
                        Value::String(_) => "string",
                        Value::Function(_) => "function",
                        Value::Node(_)
                        | Value::NodeList(_)
                        | Value::FormData(_)
                        | Value::Array(_)
                        | Value::Date(_) => "object",
                    }),
                    _ => {
                        let value = self.eval_expr(inner, env, event_param, event)?;
                        match value {
                            Value::Null => "object",
                            Value::Bool(_) => "boolean",
                            Value::Number(_) | Value::Float(_) => "number",
                            Value::Undefined => "undefined",
                            Value::String(_) => "string",
                            Value::Function(_) => "function",
                            Value::Node(_)
                            | Value::NodeList(_)
                            | Value::FormData(_)
                            | Value::Array(_)
                            | Value::Date(_) => "object",
                        }
                    }
                };
                Ok(Value::String(js_type.to_string()))
            }
            Expr::Add(parts) => {
                if parts.is_empty() {
                    return Ok(Value::String(String::new()));
                }
                let mut iter = parts.iter();
                let first = iter
                    .next()
                    .ok_or_else(|| Error::ScriptRuntime("empty add expression".into()))?;
                let mut acc = self.eval_expr(first, env, event_param, event)?;
                for part in iter {
                    let rhs = self.eval_expr(part, env, event_param, event)?;
                    acc = self.add_values(&acc, &rhs);
                }
                Ok(acc)
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
            BinaryOp::In => Value::Bool(self.value_in(left, right)),
            BinaryOp::InstanceOf => Value::Bool(self.value_instance_of(left, right)),
            BinaryOp::BitOr => {
                Value::Number(i64::from(self.to_i32_for_bitwise(left) | self.to_i32_for_bitwise(right)))
            }
            BinaryOp::BitXor => {
                Value::Number(i64::from(
                    self.to_i32_for_bitwise(left) ^ self.to_i32_for_bitwise(right),
                ))
            }
            BinaryOp::BitAnd => {
                Value::Number(i64::from(
                    self.to_i32_for_bitwise(left) & self.to_i32_for_bitwise(right),
                ))
            }
            BinaryOp::ShiftLeft => {
                let shift = self.to_u32_for_bitwise(right) & 0x1f;
                Value::Number(i64::from(self.to_i32_for_bitwise(left) << shift))
            }
            BinaryOp::ShiftRight => {
                let shift = self.to_u32_for_bitwise(right) & 0x1f;
                Value::Number(i64::from(self.to_i32_for_bitwise(left) >> shift))
            }
            BinaryOp::UnsignedShiftRight => {
                let shift = self.to_u32_for_bitwise(right) & 0x1f;
                Value::Number(i64::from(self.to_u32_for_bitwise(left) >> shift))
            }
            BinaryOp::Pow => {
                Value::Float(self.numeric_value(left).powf(self.numeric_value(right)))
            }
            BinaryOp::Lt => Value::Bool(self.compare(left, right, |l, r| l < r)),
            BinaryOp::Gt => Value::Bool(self.compare(left, right, |l, r| l > r)),
            BinaryOp::Le => Value::Bool(self.compare(left, right, |l, r| l <= r)),
            BinaryOp::Ge => Value::Bool(self.compare(left, right, |l, r| l >= r)),
            BinaryOp::Sub => Value::Float(self.numeric_value(left) - self.numeric_value(right)),
            BinaryOp::Mul => Value::Float(self.numeric_value(left) * self.numeric_value(right)),
            BinaryOp::Mod => {
                let rhs = self.numeric_value(right);
                if rhs == 0.0 {
                    return Err(Error::ScriptRuntime("modulo by zero".into()));
                }
                Value::Float(self.numeric_value(left) % rhs)
            }
            BinaryOp::Div => {
                let rhs = self.numeric_value(right);
                if rhs == 0.0 {
                    return Err(Error::ScriptRuntime("division by zero".into()));
                }
                Value::Float(self.numeric_value(left) / rhs)
            }
        };
        Ok(out)
    }

    fn value_in(&self, left: &Value, right: &Value) -> bool {
        match right {
            Value::NodeList(nodes) => self
                .value_as_index(left)
                .is_some_and(|index| index < nodes.len()),
            Value::Array(values) => self
                .value_as_index(left)
                .is_some_and(|index| index < values.borrow().len()),
            Value::FormData(entries) => {
                let key = left.as_string();
                entries.iter().any(|(name, _)| name == &key)
            }
            _ => false,
        }
    }

    fn value_instance_of(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Node(left), Value::Node(right)) => left == right,
            (Value::Node(left), Value::NodeList(nodes)) => nodes.contains(left),
            (Value::Array(left), Value::Array(right)) => Rc::ptr_eq(left, right),
            (Value::Date(left), Value::Date(right)) => Rc::ptr_eq(left, right),
            (Value::FormData(left), Value::FormData(right)) => left == right,
            _ => false,
        }
    }

    fn value_as_index(&self, value: &Value) -> Option<usize> {
        match value {
            Value::Number(v) => usize::try_from(*v).ok(),
            Value::Float(v) => {
                if !v.is_finite() || v.fract() != 0.0 || *v < 0.0 {
                    None
                } else {
                    usize::try_from(*v as i64).ok()
                }
            }
            Value::String(s) => {
                if let Ok(int) = s.parse::<i64>() {
                    usize::try_from(int).ok()
                } else if let Ok(float) = s.parse::<f64>() {
                    if float.fract() == 0.0 && float >= 0.0 {
                        usize::try_from(float as i64).ok()
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn strict_equal(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Bool(l), Value::Bool(r)) => l == r,
            (Value::Number(l), Value::Number(r)) => l == r,
            (Value::Float(l), Value::Float(r)) => l == r,
            (Value::Number(l), Value::Float(r)) => (*l as f64) == *r,
            (Value::Float(l), Value::Number(r)) => *l == (*r as f64),
            (Value::String(l), Value::String(r)) => l == r,
            (Value::Node(l), Value::Node(r)) => l == r,
            (Value::Array(l), Value::Array(r)) => Rc::ptr_eq(l, r),
            (Value::Date(l), Value::Date(r)) => Rc::ptr_eq(l, r),
            (Value::FormData(l), Value::FormData(r)) => l == r,
            (Value::Null, Value::Null) => true,
            (Value::Undefined, Value::Undefined) => true,
            _ => false,
        }
    }

    fn compare<F>(&self, left: &Value, right: &Value, op: F) -> bool
    where
        F: Fn(f64, f64) -> bool,
    {
        let l = self.numeric_value(left);
        let r = self.numeric_value(right);
        op(l, r)
    }

    fn add_values(&self, left: &Value, right: &Value) -> Value {
        if matches!(left, Value::String(_)) || matches!(right, Value::String(_)) {
            return Value::String(format!("{}{}", left.as_string(), right.as_string()));
        }

        match (left, right) {
            (Value::Number(l), Value::Number(r)) => {
                if let Some(sum) = l.checked_add(*r) {
                    Value::Number(sum)
                } else {
                    Value::Float((*l as f64) + (*r as f64))
                }
            }
            _ => Value::Float(self.numeric_value(left) + self.numeric_value(right)),
        }
    }

    fn new_array_value(values: Vec<Value>) -> Value {
        Value::Array(Rc::new(RefCell::new(values)))
    }

    fn new_date_value(timestamp_ms: i64) -> Value {
        Value::Date(Rc::new(RefCell::new(timestamp_ms)))
    }

    fn resolve_date_from_env(
        &self,
        env: &HashMap<String, Value>,
        target: &str,
    ) -> Result<Rc<RefCell<i64>>> {
        match env.get(target) {
            Some(Value::Date(value)) => Ok(value.clone()),
            Some(_) => Err(Error::ScriptRuntime(format!(
                "variable '{}' is not a Date",
                target
            ))),
            None => Err(Error::ScriptRuntime(format!(
                "unknown variable: {}",
                target
            ))),
        }
    }

    fn coerce_date_timestamp_ms(&self, value: &Value) -> i64 {
        match value {
            Value::Date(value) => *value.borrow(),
            Value::String(value) => Self::parse_date_string_to_epoch_ms(value).unwrap_or(0),
            _ => Self::value_to_i64(value),
        }
    }

    fn resolve_array_from_env(
        &self,
        env: &HashMap<String, Value>,
        target: &str,
    ) -> Result<Rc<RefCell<Vec<Value>>>> {
        match env.get(target) {
            Some(Value::Array(values)) => Ok(values.clone()),
            Some(_) => Err(Error::ScriptRuntime(format!(
                "variable '{}' is not an array",
                target
            ))),
            None => Err(Error::ScriptRuntime(format!(
                "unknown variable: {}",
                target
            ))),
        }
    }

    fn execute_array_callback(
        &mut self,
        callback: &ScriptHandler,
        args: &[Value],
        env: &HashMap<String, Value>,
        event: &EventState,
    ) -> Result<Value> {
        let mut callback_env = env.clone();
        callback_env.remove(INTERNAL_RETURN_SLOT);
        callback.bind_event_params(args, &mut callback_env);
        let mut callback_event = event.clone();
        let event_param = None;
        match self.execute_stmts(
            &callback.stmts,
            &event_param,
            &mut callback_event,
            &mut callback_env,
        )? {
            ExecFlow::Continue | ExecFlow::Return => {}
            ExecFlow::Break => {
                return Err(Error::ScriptRuntime(
                    "break statement outside of loop".into(),
                ));
            }
            ExecFlow::ContinueLoop => {
                return Err(Error::ScriptRuntime(
                    "continue statement outside of loop".into(),
                ));
            }
        }

        Ok(callback_env
            .remove(INTERNAL_RETURN_SLOT)
            .unwrap_or(Value::Undefined))
    }

    fn execute_array_callback_in_env(
        &mut self,
        callback: &ScriptHandler,
        args: &[Value],
        env: &mut HashMap<String, Value>,
        event: &EventState,
    ) -> Result<()> {
        let mut previous_values = Vec::with_capacity(callback.params.len());
        for (idx, param) in callback.params.iter().enumerate() {
            previous_values.push((param.clone(), env.get(param).cloned()));
            let value = args.get(idx).cloned().unwrap_or(Value::Undefined);
            env.insert(param.clone(), value);
        }

        let mut callback_event = event.clone();
        let event_param = None;
        let result = self.execute_stmts(
            &callback.stmts,
            &event_param,
            &mut callback_event,
            env,
        );
        env.remove(INTERNAL_RETURN_SLOT);

        for (name, previous) in previous_values {
            if let Some(previous) = previous {
                env.insert(name, previous);
            } else {
                env.remove(&name);
            }
        }

        match result? {
            ExecFlow::Continue | ExecFlow::Return => Ok(()),
            ExecFlow::Break => Err(Error::ScriptRuntime(
                "break statement outside of loop".into(),
            )),
            ExecFlow::ContinueLoop => Err(Error::ScriptRuntime(
                "continue statement outside of loop".into(),
            )),
        }
    }

    fn normalize_slice_index(len: usize, index: i64) -> usize {
        if index < 0 {
            len.saturating_sub(index.unsigned_abs() as usize)
        } else {
            (index as usize).min(len)
        }
    }

    fn normalize_splice_start_index(len: usize, start: i64) -> usize {
        if start < 0 {
            len.saturating_sub(start.unsigned_abs() as usize)
        } else {
            (start as usize).min(len)
        }
    }

    fn normalize_substring_index(len: usize, index: i64) -> usize {
        if index < 0 {
            0
        } else {
            (index as usize).min(len)
        }
    }

    fn char_index_to_byte(value: &str, char_index: usize) -> usize {
        value
            .char_indices()
            .nth(char_index)
            .map(|(idx, _)| idx)
            .unwrap_or(value.len())
    }

    fn substring_chars(value: &str, start: usize, end: usize) -> String {
        if start >= end {
            return String::new();
        }
        value.chars().skip(start).take(end - start).collect()
    }

    fn split_string(value: &str, separator: Option<String>, limit: Option<i64>) -> Vec<Value> {
        let mut parts = match separator {
            None => vec![Value::String(value.to_string())],
            Some(separator) => {
                if separator.is_empty() {
                    value
                        .chars()
                        .map(|ch| Value::String(ch.to_string()))
                        .collect::<Vec<_>>()
                } else {
                    value
                        .split(&separator)
                        .map(|part| Value::String(part.to_string()))
                        .collect::<Vec<_>>()
                }
            }
        };

        if let Some(limit) = limit {
            if limit == 0 {
                parts.clear();
            } else if limit > 0 {
                parts.truncate(limit as usize);
            }
        }

        parts
    }

    fn string_index_of(value: &str, search: &str, start_char_idx: usize) -> Option<usize> {
        let start_byte = Self::char_index_to_byte(value, start_char_idx);
        let pos = value.get(start_byte..)?.find(search)?;
        Some(value[..start_byte + pos].chars().count())
    }

    fn parse_date_string_to_epoch_ms(src: &str) -> Option<i64> {
        let src = src.trim();
        if src.is_empty() {
            return None;
        }

        let bytes = src.as_bytes();
        let mut i = 0usize;

        let mut sign = 1i64;
        if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
            if bytes[i] == b'-' {
                sign = -1;
            }
            i += 1;
        }

        let year_start = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if i <= year_start || (i - year_start) < 4 {
            return None;
        }
        let year = sign * src.get(year_start..i)?.parse::<i64>().ok()?;

        if i >= bytes.len() || bytes[i] != b'-' {
            return None;
        }
        i += 1;
        let month = Self::parse_fixed_digits_i64(src, &mut i, 2)?;
        if i >= bytes.len() || bytes[i] != b'-' {
            return None;
        }
        i += 1;
        let day = Self::parse_fixed_digits_i64(src, &mut i, 2)?;

        let month = u32::try_from(month).ok()?;
        if !(1..=12).contains(&month) {
            return None;
        }
        let day = u32::try_from(day).ok()?;
        if day == 0 || day > Self::days_in_month(year, month) {
            return None;
        }

        let mut hour = 0i64;
        let mut minute = 0i64;
        let mut second = 0i64;
        let mut millisecond = 0i64;
        let mut offset_minutes = 0i64;

        if i < bytes.len() {
            if bytes[i] != b'T' && bytes[i] != b' ' {
                return None;
            }
            i += 1;

            hour = Self::parse_fixed_digits_i64(src, &mut i, 2)?;
            if i >= bytes.len() || bytes[i] != b':' {
                return None;
            }
            i += 1;
            minute = Self::parse_fixed_digits_i64(src, &mut i, 2)?;

            if i < bytes.len() && bytes[i] == b':' {
                i += 1;
                second = Self::parse_fixed_digits_i64(src, &mut i, 2)?;
            }

            if i < bytes.len() && bytes[i] == b'.' {
                i += 1;
                let frac_start = i;
                while i < bytes.len() && bytes[i].is_ascii_digit() {
                    i += 1;
                }
                if i == frac_start {
                    return None;
                }

                let frac = src.get(frac_start..i)?;
                let mut parsed = 0i64;
                let mut digits = 0usize;
                for ch in frac.chars().take(3) {
                    parsed = parsed * 10 + i64::from(ch.to_digit(10)?);
                    digits += 1;
                }
                while digits < 3 {
                    parsed *= 10;
                    digits += 1;
                }
                millisecond = parsed;
            }

            if i < bytes.len() {
                match bytes[i] {
                    b'Z' | b'z' => {
                        i += 1;
                    }
                    b'+' | b'-' => {
                        let tz_sign = if bytes[i] == b'+' { 1 } else { -1 };
                        i += 1;
                        let tz_hour = Self::parse_fixed_digits_i64(src, &mut i, 2)?;
                        let tz_minute = if i < bytes.len() && bytes[i] == b':' {
                            i += 1;
                            Self::parse_fixed_digits_i64(src, &mut i, 2)?
                        } else {
                            Self::parse_fixed_digits_i64(src, &mut i, 2)?
                        };
                        if tz_hour > 23 || tz_minute > 59 {
                            return None;
                        }
                        offset_minutes = tz_sign * (tz_hour * 60 + tz_minute);
                    }
                    _ => return None,
                }
            }
        }

        if i != bytes.len() {
            return None;
        }
        if hour > 23 || minute > 59 || second > 59 {
            return None;
        }

        let timestamp_ms = Self::utc_timestamp_ms_from_components(
            year,
            i64::from(month) - 1,
            i64::from(day),
            hour,
            minute,
            second,
            millisecond,
        );
        Some(timestamp_ms - offset_minutes * 60_000)
    }

    fn parse_fixed_digits_i64(src: &str, i: &mut usize, width: usize) -> Option<i64> {
        let end = i.checked_add(width)?;
        let segment = src.get(*i..end)?;
        if !segment.as_bytes().iter().all(|b| b.is_ascii_digit()) {
            return None;
        }
        *i = end;
        segment.parse::<i64>().ok()
    }

    fn format_iso_8601_utc(timestamp_ms: i64) -> String {
        let (year, month, day, hour, minute, second, millisecond) =
            Self::date_components_utc(timestamp_ms);
        let year_str = if (0..=9999).contains(&year) {
            format!("{year:04}")
        } else if year < 0 {
            format!("-{:06}", -(year as i128))
        } else {
            format!("+{:06}", year)
        };
        format!(
            "{year_str}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{millisecond:03}Z"
        )
    }

    fn date_components_utc(timestamp_ms: i64) -> (i64, u32, u32, u32, u32, u32, u32) {
        let days = timestamp_ms.div_euclid(86_400_000);
        let rem = timestamp_ms.rem_euclid(86_400_000);
        let hour = (rem / 3_600_000) as u32;
        let minute = ((rem % 3_600_000) / 60_000) as u32;
        let second = ((rem % 60_000) / 1_000) as u32;
        let millisecond = (rem % 1_000) as u32;
        let (year, month, day) = Self::civil_from_days(days);
        (year, month, day, hour, minute, second, millisecond)
    }

    fn utc_timestamp_ms_from_components(
        year: i64,
        month_zero_based: i64,
        day: i64,
        hour: i64,
        minute: i64,
        second: i64,
        millisecond: i64,
    ) -> i64 {
        let (norm_year, norm_month) = Self::normalize_year_month(year, month_zero_based);
        let mut days = Self::days_from_civil(norm_year, norm_month, 1) + (day - 1);
        let mut time_ms = ((hour * 60 + minute) * 60 + second) * 1_000 + millisecond;
        days += time_ms.div_euclid(86_400_000);
        time_ms = time_ms.rem_euclid(86_400_000);

        let out = (days as i128) * 86_400_000i128 + (time_ms as i128);
        out.clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64
    }

    fn normalize_year_month(year: i64, month_zero_based: i64) -> (i64, u32) {
        let total_month = year.saturating_mul(12).saturating_add(month_zero_based);
        let norm_year = total_month.div_euclid(12);
        let norm_month = total_month.rem_euclid(12) as u32 + 1;
        (norm_year, norm_month)
    }

    fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
        let adjusted_year = year - if month <= 2 { 1 } else { 0 };
        let era = adjusted_year.div_euclid(400);
        let yoe = adjusted_year - era * 400;
        let month = i64::from(month);
        let day = i64::from(day);
        let doy = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146_097 + doe - 719_468
    }

    fn civil_from_days(days: i64) -> (i64, u32, u32) {
        let z = days + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096).div_euclid(365);
        let mut year = yoe + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2).div_euclid(153);
        let day = (doy - (153 * mp + 2).div_euclid(5) + 1) as u32;
        let month = (mp + if mp < 10 { 3 } else { -9 }) as u32;
        if month <= 2 {
            year += 1;
        }
        (year, month, day)
    }

    fn days_in_month(year: i64, month: u32) -> u32 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if Self::is_leap_year(year) {
                    29
                } else {
                    28
                }
            }
            _ => 0,
        }
    }

    fn is_leap_year(year: i64) -> bool {
        (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
    }

    fn numeric_value(&self, value: &Value) -> f64 {
        match value {
            Value::Number(v) => *v as f64,
            Value::Float(v) => *v,
            Value::Date(v) => *v.borrow() as f64,
            Value::Null => 0.0,
            Value::Undefined => f64::NAN,
            _ => value.as_string().parse::<f64>().unwrap_or(0.0),
        }
    }

    fn coerce_number_for_global(value: &Value) -> f64 {
        match value {
            Value::Number(v) => *v as f64,
            Value::Float(v) => *v,
            Value::Bool(v) => {
                if *v {
                    1.0
                } else {
                    0.0
                }
            }
            Value::Null => 0.0,
            Value::Undefined => f64::NAN,
            Value::String(v) => {
                let trimmed = v.trim();
                if trimmed.is_empty() {
                    0.0
                } else {
                    trimmed.parse::<f64>().unwrap_or(f64::NAN)
                }
            }
            Value::Date(v) => *v.borrow() as f64,
            Value::Node(_) | Value::NodeList(_) | Value::FormData(_) | Value::Function(_) => {
                f64::NAN
            }
            Value::Array(values) => {
                let rendered = Value::Array(values.clone()).as_string();
                let trimmed = rendered.trim();
                if trimmed.is_empty() {
                    0.0
                } else {
                    trimmed.parse::<f64>().unwrap_or(f64::NAN)
                }
            }
        }
    }

    fn to_i32_for_bitwise(&self, value: &Value) -> i32 {
        let numeric = self.numeric_value(value);
        if !numeric.is_finite() {
            return 0;
        }
        let unsigned = numeric.trunc().rem_euclid(4_294_967_296.0);
        if unsigned >= 2_147_483_648.0 {
            (unsigned - 4_294_967_296.0) as i32
        } else {
            unsigned as i32
        }
    }

    fn to_u32_for_bitwise(&self, value: &Value) -> u32 {
        let numeric = self.numeric_value(value);
        if !numeric.is_finite() {
            return 0;
        }
        numeric.trunc().rem_euclid(4_294_967_296.0) as u32
    }

    fn resolve_dom_query_list_static(&mut self, target: &DomQuery) -> Result<Option<Vec<NodeId>>> {
        match target {
            DomQuery::BySelectorAll { selector } => {
                Ok(Some(self.dom.query_selector_all(selector)?))
            }
            DomQuery::QuerySelectorAll { target, selector } => {
                let Some(target_node) = self.resolve_dom_query_static(target)? else {
                    return Ok(None);
                };
                Ok(Some(
                    self.dom.query_selector_all_from(&target_node, selector)?,
                ))
            }
            DomQuery::Index { target, index } => {
                let Some(list) = self.resolve_dom_query_list_static(target)? else {
                    return Ok(None);
                };
                let index = self.resolve_runtime_dom_index(index, None)?;
                Ok(list.get(index).copied().map(|node| vec![node]))
            }
            DomQuery::BySelectorAllIndex { selector, index } => {
                let index = index
                    .static_index()
                    .ok_or_else(|| Error::ScriptRuntime("dynamic index in static context".into()))?;
                Ok(self
                    .dom
                    .query_selector_all(selector)?
                    .get(index)
                    .copied()
                    .map(|node| vec![node]))
            }
            DomQuery::QuerySelectorAllIndex {
                target,
                selector,
                index,
            } => {
                let Some(target_node) = self.resolve_dom_query_static(target)? else {
                    return Ok(None);
                };
                let index = index
                    .static_index()
                    .ok_or_else(|| Error::ScriptRuntime("dynamic index in static context".into()))?;
                let list = self.dom.query_selector_all_from(&target_node, selector)?;
                Ok(list.get(index).copied().map(|node| vec![node]))
            }
            DomQuery::Var(_) => Err(Error::ScriptRuntime(
                "element variable cannot be resolved in static context".into(),
            )),
            _ => Ok(None),
        }
    }

    fn resolve_dom_query_list_runtime(
        &mut self,
        target: &DomQuery,
        env: &HashMap<String, Value>,
    ) -> Result<Option<Vec<NodeId>>> {
        match target {
            DomQuery::Var(name) => match env.get(name) {
                Some(Value::NodeList(nodes)) => Ok(Some(nodes.clone())),
                Some(Value::Node(_)) => Err(Error::ScriptRuntime(format!(
                    "variable '{}' is not a node list",
                    name
                ))),
                Some(_) => Err(Error::ScriptRuntime(format!(
                    "variable '{}' is not a node list",
                    name
                ))),
                None => Err(Error::ScriptRuntime(format!(
                    "unknown element variable: {}",
                    name
                ))),
            },
            _ => self.resolve_dom_query_list_static(target),
        }
    }

    fn resolve_dom_query_static(&mut self, target: &DomQuery) -> Result<Option<NodeId>> {
        match target {
            DomQuery::DocumentRoot => Ok(Some(self.dom.root)),
            DomQuery::ById(id) => Ok(self.dom.by_id(id)),
            DomQuery::BySelector(selector) => self.dom.query_selector(selector),
            DomQuery::BySelectorAll { .. } => Err(Error::ScriptRuntime(
                "cannot use querySelectorAll result as single element".into(),
            )),
            DomQuery::BySelectorAllIndex { selector, index } => {
                let index = index
                    .static_index()
                    .ok_or_else(|| Error::ScriptRuntime("dynamic index in static context".into()))?;
                let all = self.dom.query_selector_all(selector)?;
                Ok(all.get(index).copied())
            }
            DomQuery::Index { target, index } => {
                let Some(list) = self.resolve_dom_query_list_static(target)? else {
                    return Ok(None);
                };
                let index = self.resolve_runtime_dom_index(index, None)?;
                Ok(list.get(index).copied())
            }
            DomQuery::QuerySelector { target, selector } => {
                let Some(target_node) = self.resolve_dom_query_static(target)? else {
                    return Ok(None);
                };
                self.dom.query_selector_from(&target_node, selector)
            }
            DomQuery::QuerySelectorAll { .. } => Err(Error::ScriptRuntime(
                "cannot use querySelectorAll result as single element".into(),
            )),
            DomQuery::QuerySelectorAllIndex {
                target,
                selector,
                index,
            } => {
                let Some(target_node) = self.resolve_dom_query_static(target)? else {
                    return Ok(None);
                };
                let index = index
                    .static_index()
                    .ok_or_else(|| Error::ScriptRuntime("dynamic index in static context".into()))?;
                let all = self.dom.query_selector_all_from(&target_node, selector)?;
                Ok(all.get(index).copied())
            }
            DomQuery::FormElementsIndex { form, index } => {
                let Some(form_node) = self.resolve_dom_query_static(form)? else {
                    return Ok(None);
                };
                let all = self.form_elements(form_node)?;
                let index = self.resolve_runtime_dom_index(index, None)?;
                Ok(all.get(index).copied())
            }
            DomQuery::Var(_) => Err(Error::ScriptRuntime(
                "element variable cannot be resolved in static context".into(),
            )),
        }
    }

    fn resolve_dom_query_runtime(
        &mut self,
        target: &DomQuery,
        env: &HashMap<String, Value>,
    ) -> Result<Option<NodeId>> {
        match target {
            DomQuery::DocumentRoot => Ok(Some(self.dom.root)),
            DomQuery::Var(name) => match env.get(name) {
                Some(Value::Node(node)) => Ok(Some(*node)),
                Some(Value::NodeList(_)) => Err(Error::ScriptRuntime(format!(
                    "variable '{}' is not a single element",
                    name
                ))),
                Some(_) => Err(Error::ScriptRuntime(format!(
                    "variable '{}' is not a single element",
                    name
                ))),
                None => Err(Error::ScriptRuntime(format!(
                    "unknown element variable: {}",
                    name
                ))),
            },
            DomQuery::BySelectorAll { .. } => Err(Error::ScriptRuntime(
                "cannot use querySelectorAll result as single element".into(),
            )),
            DomQuery::QuerySelectorAll { .. } => Err(Error::ScriptRuntime(
                "cannot use querySelectorAll result as single element".into(),
            )),
            DomQuery::Index { target, index } => {
                let Some(list) = self.resolve_dom_query_list_runtime(target, env)? else {
                    return Ok(None);
                };
                let index = self.resolve_runtime_dom_index(index, Some(env))?;
                Ok(list.get(index).copied())
            }
            DomQuery::QuerySelector { target, selector } => {
                let Some(target_node) = self.resolve_dom_query_runtime(target, env)? else {
                    return Ok(None);
                };
                self.dom.query_selector_from(&target_node, selector)
            }
            DomQuery::QuerySelectorAllIndex {
                target,
                selector,
                index,
            } => {
                let Some(target_node) = self.resolve_dom_query_runtime(target, env)? else {
                    return Ok(None);
                };
                let index = self.resolve_runtime_dom_index(index, Some(env))?;
                let all = self.dom.query_selector_all_from(&target_node, selector)?;
                Ok(all.get(index).copied())
            }
            DomQuery::FormElementsIndex { form, index } => {
                let Some(form_node) = self.resolve_dom_query_runtime(form, env)? else {
                    return Ok(None);
                };
                let all = self.form_elements(form_node)?;
                let index = self.resolve_runtime_dom_index(index, Some(env))?;
                Ok(all.get(index).copied())
            }
            _ => self.resolve_dom_query_static(target),
        }
    }

    fn resolve_dom_query_required_runtime(
        &mut self,
        target: &DomQuery,
        env: &HashMap<String, Value>,
    ) -> Result<NodeId> {
        self.resolve_dom_query_runtime(target, env)?.ok_or_else(|| {
            Error::ScriptRuntime(format!("{} returned null", target.describe_call()))
        })
    }

    fn resolve_runtime_dom_index(
        &mut self,
        index: &DomIndex,
        env: Option<&HashMap<String, Value>>,
    ) -> Result<usize> {
        match index {
            DomIndex::Static(index) => Ok(*index),
            DomIndex::Dynamic(expr_src) => {
                let expr = parse_expr(expr_src)?;
                let event = EventState::new("script", self.dom.root, self.now_ms);
                let value = self.eval_expr(&expr, env.ok_or_else(|| {
                    Error::ScriptRuntime("dynamic index requires runtime context".into())
                })?, &None, &event)?;
                self.value_as_index(&value).ok_or_else(|| {
                    Error::ScriptRuntime(format!("invalid index expression: {expr_src}"))
                })
            }
        }
    }

    fn describe_dom_prop(&self, prop: &DomProp) -> String {
        match prop {
            DomProp::Value => "value".into(),
            DomProp::Checked => "checked".into(),
            DomProp::Readonly => "readonly".into(),
            DomProp::Required => "required".into(),
            DomProp::Disabled => "disabled".into(),
            DomProp::TextContent => "textContent".into(),
            DomProp::InnerHtml => "innerHTML".into(),
            DomProp::ClassName => "className".into(),
            DomProp::Id => "id".into(),
            DomProp::Name => "name".into(),
            DomProp::OffsetWidth => "offsetWidth".into(),
            DomProp::OffsetHeight => "offsetHeight".into(),
            DomProp::OffsetLeft => "offsetLeft".into(),
            DomProp::OffsetTop => "offsetTop".into(),
            DomProp::ScrollWidth => "scrollWidth".into(),
            DomProp::ScrollHeight => "scrollHeight".into(),
            DomProp::ScrollLeft => "scrollLeft".into(),
            DomProp::ScrollTop => "scrollTop".into(),
            DomProp::Dataset(_) => "dataset".into(),
            DomProp::Style(_) => "style".into(),
            DomProp::ActiveElement => "activeElement".into(),
        }
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

    fn trace_node_label(&self, node: NodeId) -> String {
        if let Some(id) = self.dom.attr(node, "id") {
            if !id.is_empty() {
                return format!("#{id}");
            }
        }
        self.dom
            .tag_name(node)
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| format!("node-{}", node.0))
    }

    fn value_to_i64(value: &Value) -> i64 {
        match value {
            Value::Number(v) => *v,
            Value::Float(v) => *v as i64,
            Value::Bool(v) => {
                if *v {
                    1
                } else {
                    0
                }
            }
            Value::String(v) => v
                .parse::<i64>()
                .ok()
                .or_else(|| v.parse::<f64>().ok().map(|n| n as i64))
                .unwrap_or(0),
            Value::Array(values) => Value::Array(values.clone())
                .as_string()
                .parse::<i64>()
                .ok()
                .or_else(|| {
                    Value::Array(values.clone())
                        .as_string()
                        .parse::<f64>()
                        .ok()
                        .map(|n| n as i64)
                })
                .unwrap_or(0),
            Value::Date(value) => *value.borrow(),
            Value::Node(_) => 0,
            Value::NodeList(_) => 0,
            Value::FormData(_) => 0,
            Value::Function(_) => 0,
            Value::Null => 0,
            Value::Undefined => 0,
        }
    }

    fn next_random_f64(&mut self) -> f64 {
        // xorshift64*: simple deterministic PRNG for test runtime.
        let mut x = self.rng_state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.rng_state = if x == 0 { 0xA5A5_A5A5_A5A5_A5A5 } else { x };
        let out = x.wrapping_mul(0x2545_F491_4F6C_DD1D);
        // Convert top 53 bits to [0.0, 1.0).
        let mantissa = out >> 11;
        (mantissa as f64) * (1.0 / ((1u64 << 53) as f64))
    }

    fn schedule_timeout(
        &mut self,
        callback: TimerCallback,
        delay_ms: i64,
        callback_args: Vec<Value>,
        env: &HashMap<String, Value>,
    ) -> i64 {
        let delay_ms = delay_ms.max(0);
        let due_at = self.now_ms + delay_ms;
        let id = self.next_timer_id;
        self.next_timer_id += 1;
        let order = self.next_task_order;
        self.next_task_order += 1;
        self.task_queue.push(ScheduledTask {
            id,
            due_at,
            order,
            interval_ms: None,
            callback,
            callback_args,
            env: env.clone(),
        });
        self.trace_timer_line(format!(
            "[timer] schedule timeout id={} due_at={} delay_ms={}",
            id, due_at, delay_ms
        ));
        id
    }

    fn schedule_interval(
        &mut self,
        callback: TimerCallback,
        interval_ms: i64,
        callback_args: Vec<Value>,
        env: &HashMap<String, Value>,
    ) -> i64 {
        let interval_ms = interval_ms.max(0);
        let due_at = self.now_ms + interval_ms;
        let id = self.next_timer_id;
        self.next_timer_id += 1;
        let order = self.next_task_order;
        self.next_task_order += 1;
        self.task_queue.push(ScheduledTask {
            id,
            due_at,
            order,
            interval_ms: Some(interval_ms),
            callback,
            callback_args,
            env: env.clone(),
        });
        self.trace_timer_line(format!(
            "[timer] schedule interval id={} due_at={} interval_ms={}",
            id, due_at, interval_ms
        ));
        id
    }

    fn clear_timeout(&mut self, id: i64) {
        let before = self.task_queue.len();
        self.task_queue.retain(|task| task.id != id);
        let removed = before.saturating_sub(self.task_queue.len());
        let mut running_canceled = false;
        if self.running_timer_id == Some(id) {
            self.running_timer_canceled = true;
            running_canceled = true;
        }
        self.trace_timer_line(format!(
            "[timer] clear id={} removed={} running_canceled={}",
            id, removed, running_canceled
        ));
    }

    fn compile_and_register_script(&mut self, script: &str) -> Result<()> {
        let stmts = parse_block_statements(script)?;
        let mut event = EventState::new("script", self.dom.root, self.now_ms);
        let mut env = self.script_env.clone();
        self.run_in_task_context(|this| {
            this.execute_stmts(&stmts, &None, &mut event, &mut env)
                .map(|_| ())
        })?;
        self.script_env = env;

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ListenerRegistrationOp {
    Add,
    Remove,
}

fn parse_element_target(cursor: &mut Cursor<'_>) -> Result<DomQuery> {
    cursor.skip_ws();
    let start = cursor.pos();
    let mut target = if let Ok(target) = parse_form_elements_item_target(cursor) {
        target
    } else {
        cursor.set_pos(start);
        parse_document_or_var_target(cursor)?
    };

    loop {
        cursor.skip_ws();
        let dot_pos = cursor.pos();
        if !cursor.consume_byte(b'.') {
            break;
        }

        cursor.skip_ws();
        let method = match cursor.parse_identifier() {
            Some(method) => method,
            None => {
                cursor.set_pos(dot_pos);
                break;
            }
        };

        match method.as_str() {
            "querySelector" => {
                cursor.skip_ws();
                cursor.expect_byte(b'(')?;
                cursor.skip_ws();
                let selector = cursor.parse_string_literal()?;
                cursor.skip_ws();
                cursor.expect_byte(b')')?;
                cursor.skip_ws();
                target = DomQuery::QuerySelector {
                    target: Box::new(target),
                    selector,
                };
            }
            "querySelectorAll" => {
                cursor.skip_ws();
                cursor.expect_byte(b'(')?;
                cursor.skip_ws();
                let selector = cursor.parse_string_literal()?;
                cursor.skip_ws();
                cursor.expect_byte(b')')?;
                cursor.skip_ws();
                target = DomQuery::QuerySelectorAll {
                    target: Box::new(target),
                    selector,
                };
            }
            _ => {
                cursor.set_pos(dot_pos);
                break;
            }
        }
    }

    loop {
        cursor.skip_ws();
        let index_pos = cursor.pos();
        if !cursor.consume_byte(b'[') {
            break;
        }

        cursor.skip_ws();
        let index_src = match cursor.read_until_byte(b']') {
            Ok(index_src) => index_src,
            Err(_) => {
                cursor.set_pos(index_pos);
                break;
            }
        };
        cursor.skip_ws();
        cursor.expect_byte(b']')?;
        let index = parse_dom_query_index(&index_src)?;
        target = match target {
            DomQuery::BySelectorAll { selector } => DomQuery::BySelectorAllIndex { selector, index },
            DomQuery::QuerySelectorAll { target, selector } => DomQuery::QuerySelectorAllIndex {
                target,
                selector,
                index,
            },
            _ => DomQuery::Index {
                target: Box::new(target),
                index,
            },
        };
        cursor.skip_ws();
    }
    Ok(target)
}

fn parse_document_or_var_target(cursor: &mut Cursor<'_>) -> Result<DomQuery> {
    let start = cursor.pos();
    if let Ok(target) = parse_document_element_call(cursor) {
        return Ok(target);
    }
    cursor.set_pos(start);
    if cursor.consume_ascii("document") {
        return Ok(DomQuery::DocumentRoot);
    }
    cursor.set_pos(start);
    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if cursor.consume_byte(b'.') {
            cursor.skip_ws();
            if cursor.consume_ascii("document") {
                cursor.skip_ws();
            }
        }
        return Ok(DomQuery::DocumentRoot);
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

fn parse_form_elements_item_target(cursor: &mut Cursor<'_>) -> Result<DomQuery> {
    let form = parse_form_elements_base(cursor)?;
        cursor.skip_ws();
        cursor.expect_byte(b'.')?;
        cursor.skip_ws();
        cursor.expect_ascii("elements")?;
        cursor.skip_ws();
        cursor.expect_byte(b'[')?;
        cursor.skip_ws();
        let index_src = cursor.read_until_byte(b']')?;
        cursor.skip_ws();
        cursor.expect_byte(b']')?;
        let index = parse_dom_query_index(&index_src)?;
        Ok(DomQuery::FormElementsIndex {
            form: Box::new(form),
            index,
        })
}

fn parse_dom_query_index(src: &str) -> Result<DomIndex> {
    let src = strip_js_comments(src).trim().to_string();
    if src.is_empty() {
        return Err(Error::ScriptParse("empty index".into()));
    }

    let expr = parse_expr(&src)?;
    if let Expr::Number(index) = expr {
        return usize::try_from(index)
            .map(DomIndex::Static)
            .map_err(|_| Error::ScriptParse(format!("invalid index: {src}")));
    }

    Ok(DomIndex::Dynamic(src))
}

fn parse_form_elements_base(cursor: &mut Cursor<'_>) -> Result<DomQuery> {
    let start = cursor.pos();
    if let Ok(target) = parse_document_element_call(cursor) {
        return Ok(target);
    }
    cursor.set_pos(start);
    if let Some(name) = cursor.parse_identifier() {
        return Ok(DomQuery::Var(name));
    }
    Err(Error::ScriptParse(format!(
        "expected form target at {}",
        start
    )))
}

fn parse_document_element_call(cursor: &mut Cursor<'_>) -> Result<DomQuery> {
    cursor.skip_ws();
    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        cursor.expect_byte(b'.')?;
        cursor.skip_ws();
    }
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
        "querySelectorAll" => Ok(DomQuery::BySelectorAll { selector: arg }),
        "getElementsByTagName" => Ok(DomQuery::BySelectorAll {
            selector: normalize_get_elements_by_tag_name(&arg)?,
        }),
        "getElementsByClassName" => Ok(DomQuery::BySelectorAll {
            selector: normalize_get_elements_by_class_name(&arg)?,
        }),
        "getElementsByName" => Ok(DomQuery::BySelectorAll {
            selector: normalize_get_elements_by_name(&arg)?,
        }),
        _ => Err(Error::ScriptParse(format!(
            "unsupported document method: {}",
            method
        ))),
    }
}

fn normalize_get_elements_by_tag_name(tag_name: &str) -> Result<String> {
    let tag_name = tag_name.trim();
    if tag_name.is_empty() {
        return Err(Error::ScriptParse(
            "getElementsByTagName requires a tag name".into(),
        ));
    }
    if tag_name == "*" {
        return Ok("*".into());
    }
    Ok(tag_name.to_ascii_lowercase())
}

fn normalize_get_elements_by_class_name(class_names: &str) -> Result<String> {
    let mut selector = String::new();
    let classes: Vec<&str> = class_names
        .split_whitespace()
        .map(str::trim)
        .filter(|class_name| !class_name.is_empty())
        .collect();

    if classes.is_empty() {
        return Err(Error::ScriptParse(
            "getElementsByClassName requires at least one class name".into(),
        ));
    }

    for class_name in classes {
        selector.push('.');
        selector.push_str(class_name);
    }
    Ok(selector)
}

fn normalize_get_elements_by_name(name: &str) -> Result<String> {
    let name = name.trim();
    if name.is_empty() {
        return Err(Error::ScriptParse(
            "getElementsByName requires a name value".into(),
        ));
    }
    let escaped = name.replace('\\', "\\\\").replace('\'', "\\'");
    Ok(format!("[name='{}']", escaped))
}

fn parse_callback_parameter_list(
    src: &str,
    max_params: usize,
    label: &str,
) -> Result<Vec<String>> {
    let parts = split_top_level_by_char(src.trim(), b',');
    if parts.len() == 1 && parts[0].trim().is_empty() {
        return Ok(Vec::new());
    }

    if parts.len() > max_params {
        return Err(Error::ScriptParse(format!(
            "unsupported {label}: {src}"
        )));
    }

    let mut params = Vec::new();
    for raw in parts {
        let param = raw.trim();
        if !is_ident(param) {
            return Err(Error::ScriptParse(format!("unsupported {label}: {src}")));
        }
        params.push(param.to_string());
    }

    Ok(params)
}

fn parse_arrow_or_block_body(cursor: &mut Cursor<'_>) -> Result<String> {
    cursor.skip_ws();
    if cursor.peek() == Some(b'{') {
        return cursor.read_balanced_block(b'{', b'}');
    }

    let src = cursor
        .src
        .get(cursor.i..)
        .ok_or_else(|| Error::ScriptParse("expected callback body".into()))?;
    let mut end = src.len();

    while end > 0 {
        let raw = src.get(0..end).ok_or_else(|| Error::ScriptParse("invalid callback body".into()))?;
        let expr_src = raw.trim();
        if expr_src.is_empty() {
            break;
        }

        let stripped = strip_js_comments(expr_src);
        let stripped = stripped.trim();
        if !stripped.is_empty() {
            if parse_expr(stripped).is_ok() {
                cursor.set_pos(cursor.i + expr_src.len());
                return Ok(stripped.to_string());
            }
        }

        end -= 1;
    }

    Err(Error::ScriptParse("expected callback body".into()))
}

fn parse_function_expr(src: &str) -> Result<Option<Expr>> {
    let src = src.trim();
    if !src.starts_with("function") && !src.contains("=>") {
        return Ok(None);
    }

    let mut cursor = Cursor::new(src);
    let parsed = match parse_callback(&mut cursor, usize::MAX, "function parameters") {
        Ok(parsed) => parsed,
        Err(err) => {
            if src.starts_with("function") {
                return Err(err);
            }
            return Ok(None);
        }
    };

    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    let (params, body) = parsed;
    let stmts = parse_block_statements(&body)?;
    Ok(Some(Expr::Function {
        handler: ScriptHandler {
            params,
            stmts,
        },
    }))
}

fn parse_callback(
    cursor: &mut Cursor<'_>,
    max_params: usize,
    label: &str,
) -> Result<(Vec<String>, String)> {
    cursor.skip_ws();

    let params = if cursor
        .src
        .get(cursor.i..)
        .is_some_and(|src| src.starts_with("function"))
        && !cursor
            .bytes()
            .get(cursor.i + "function".len())
            .is_some_and(|&b| is_ident_char(b))
    {
        cursor.consume_ascii("function");
        cursor.skip_ws();

        if !cursor.consume_byte(b'(') {
            let _ = cursor
                .parse_identifier()
                .ok_or_else(|| Error::ScriptParse("expected function name".into()))?;
            cursor.skip_ws();
            cursor.expect_byte(b'(')?;
        }

        let params = cursor.read_until_byte(b')')?;
        cursor.expect_byte(b')')?;
        let params = parse_callback_parameter_list(&params, max_params, label)?;
        cursor.skip_ws();
        let body = cursor.read_balanced_block(b'{', b'}')?;
        return Ok((params, body));
    } else if cursor.consume_byte(b'(') {
        let params = cursor.read_until_byte(b')')?;
        cursor.expect_byte(b')')?;
        let params = parse_callback_parameter_list(&params, max_params, label)?;
        params
    } else {
        let ident = cursor
            .parse_identifier()
            .ok_or_else(|| Error::ScriptParse("expected callback parameter or ()".into()))?;
        vec![ident]
    };

    cursor.skip_ws();
    cursor.expect_ascii("=>")?;
    let body = parse_arrow_or_block_body(cursor)?;
    Ok((params, body))
}

fn parse_timer_callback(
    timer_name: &str,
    src: &str,
) -> Result<TimerCallback> {
    let mut cursor = Cursor::new(src);
    if let Ok((params, body)) = parse_callback(&mut cursor, usize::MAX, "timer callback parameters") {
        cursor.skip_ws();
        if cursor.eof() {
            return Ok(TimerCallback::Inline(ScriptHandler {
                params,
                stmts: parse_block_statements(&body)?,
            }));
        }
    }

    match parse_expr(src)? {
        Expr::Function { handler } => Ok(TimerCallback::Inline(handler)),
        Expr::Var(name) => Ok(TimerCallback::Reference(name)),
        _ => Err(Error::ScriptParse(format!(
            "unsupported {timer_name} callback: {src}"
        ))),
    }
}

fn parse_block_statements(body: &str) -> Result<Vec<Stmt>> {
    let sanitized = strip_js_comments(body);
    let raw_stmts = split_top_level_statements(sanitized.as_str());
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

    if let Some(parsed) = parse_do_while_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_while_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_for_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_return_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_break_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_continue_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_query_selector_all_foreach_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_array_for_each_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_var_decl(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_var_assign(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_update_stmt(stmt) {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_form_data_append_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_dom_method_call_stmt(stmt)? {
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

    if let Some(parsed) = parse_insert_adjacent_element_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_insert_adjacent_text_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_insert_adjacent_html_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_set_timeout_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_set_interval_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_queue_microtask_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_clear_timeout_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_node_tree_mutation_stmt(stmt)? {
        return Ok(parsed);
    }

    if let Some(parsed) = parse_node_remove_stmt(stmt)? {
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

fn parse_while_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let mut cursor = Cursor::new(stmt);
    cursor.skip_ws();

    if !cursor.consume_ascii("while") {
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

    cursor.skip_ws();
    let body_src = cursor.read_balanced_block(b'{', b'}')?;
    let body = parse_block_statements(&body_src)?;

    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported while statement tail: {stmt}"
        )));
    }

    Ok(Some(Stmt::While { cond, body }))
}

fn parse_do_while_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let mut cursor = Cursor::new(stmt);
    cursor.skip_ws();

    if !cursor.consume_ascii("do") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }

    cursor.skip_ws();
    let body_src = cursor.read_balanced_block(b'{', b'}')?;
    let body = parse_block_statements(&body_src)?;

    cursor.skip_ws();
    if !cursor.consume_ascii("while") {
        return Err(Error::ScriptParse(format!("unsupported do statement: {stmt}")));
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Err(Error::ScriptParse(format!("unsupported do statement: {stmt}")));
        }
    }

    cursor.skip_ws();
    let cond_src = cursor.read_balanced_block(b'(', b')')?;
    let cond = parse_expr(cond_src.trim())?;

    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!("unsupported do while statement tail: {stmt}")));
    }

    Ok(Some(Stmt::DoWhile { cond, body }))
}

fn parse_return_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let mut cursor = Cursor::new(stmt);
    cursor.skip_ws();
    if !cursor.consume_ascii("return") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }

    cursor.skip_ws();
    if cursor.eof() {
        return Ok(Some(Stmt::Return { value: None }));
    }

    let expr_src = cursor.src.get(cursor.i..).unwrap_or_default().trim();
    let expr_src = expr_src.strip_suffix(';').unwrap_or(expr_src).trim();
    if expr_src.is_empty() {
        return Ok(Some(Stmt::Return { value: None }));
    }
    let value = parse_expr(expr_src)?;
    Ok(Some(Stmt::Return { value: Some(value) }))
}

fn parse_break_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let mut cursor = Cursor::new(stmt);
    cursor.skip_ws();
    if !cursor.consume_ascii("break") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!("unsupported break statement: {stmt}")));
    }
    Ok(Some(Stmt::Break))
}

fn parse_continue_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let mut cursor = Cursor::new(stmt);
    cursor.skip_ws();
    if !cursor.consume_ascii("continue") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported continue statement: {stmt}"
        )));
    }
    Ok(Some(Stmt::Continue))
}

fn parse_for_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let mut cursor = Cursor::new(stmt);
    cursor.skip_ws();

    if !cursor.consume_ascii("for") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }

    cursor.skip_ws();
    let header_src = cursor.read_balanced_block(b'(', b')')?;
    if let Some((kind, item_var, iterable_src)) = parse_for_in_of_stmt(&header_src)? {
        let iterable = parse_expr(iterable_src.trim())?;
        cursor.skip_ws();
        let body_src = cursor.read_balanced_block(b'{', b'}')?;
        let body = parse_block_statements(&body_src)?;

        cursor.skip_ws();
        cursor.consume_byte(b';');
        cursor.skip_ws();
        if !cursor.eof() {
            return Err(Error::ScriptParse(format!(
                "unsupported for statement tail: {stmt}"
            )));
        }

        let stmt = match kind {
            ForInOfKind::In => Stmt::ForIn {
                item_var,
                iterable,
                body,
            },
            ForInOfKind::Of => Stmt::ForOf {
                item_var,
                iterable,
                body,
            },
        };
        return Ok(Some(stmt));
    }

    let header_parts = split_top_level_by_char(header_src.trim(), b';');
    if header_parts.len() != 3 {
        return Err(Error::ScriptParse(format!("unsupported for statement: {stmt}")));
    }

    let init = parse_for_clause_stmt(header_parts[0])?;
    let cond = if header_parts[1].trim().is_empty() {
        None
    } else {
        Some(parse_expr(header_parts[1].trim())?)
    };
    let post = parse_for_clause_stmt(header_parts[2])?;

    cursor.skip_ws();
    let body_src = cursor.read_balanced_block(b'{', b'}')?;
    let body = parse_block_statements(&body_src)?;

    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!("unsupported for statement tail: {stmt}")));
    }

    Ok(Some(Stmt::For {
        init,
        cond,
        post,
        body,
    }))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ForInOfKind {
    In,
    Of,
}

fn parse_for_in_of_stmt(header: &str) -> Result<Option<(ForInOfKind, String, &str)>> {
    let header = header.trim();
    if header.is_empty() {
        return Ok(None);
    }

    let mut found = None;
    for (kind, keyword) in [(ForInOfKind::In, "in"), (ForInOfKind::Of, "of")] {
        if let Some(pos) = find_top_level_in_of_keyword(header, keyword)? {
            found = Some((kind, pos, keyword));
            break;
        }
    }

    let Some((kind, pos, keyword)) = found else {
        return Ok(None);
    };

    let left = header[..pos].trim();
    let right = header[pos + keyword.len()..].trim();
    if left.is_empty() || right.is_empty() {
        return Err(Error::ScriptParse(format!(
            "unsupported for statement: {header}"
        )));
    }

    let item_var = parse_for_in_of_var(left)?;
    Ok(Some((kind, item_var, right)))
}

fn find_top_level_in_of_keyword(src: &str, keyword: &str) -> Result<Option<usize>> {
    let bytes = src.as_bytes();
    let mut state = 0u8;
    let mut i = 0usize;
    let mut paren = 0isize;
    let mut bracket = 0isize;
    let mut brace = 0isize;

    while i < bytes.len() {
        let b = bytes[i];
        match state {
            0 => match b {
                b'\'' => state = 1,
                b'"' => state = 2,
                b'`' => state = 3,
                b'(' => paren += 1,
                b')' => paren -= 1,
                b'[' => bracket += 1,
                b']' => bracket -= 1,
                b'{' => brace += 1,
                b'}' => brace -= 1,
                _ => {
                    if paren == 0 && bracket == 0 && brace == 0 {
                        if i + keyword.len() <= bytes.len()
                            && &src[i..i + keyword.len()] == keyword
                        {
                            let prev_ok = i == 0
                                || !is_ident_char(src
                                    .as_bytes()
                                    .get(i.wrapping_sub(1))
                                    .copied()
                                    .unwrap_or_default());
                            let next = src.as_bytes().get(i + keyword.len()).copied();
                            let next_ok = next.is_none() || !is_ident_char(next.unwrap());
                            if prev_ok && next_ok {
                                return Ok(Some(i));
                            }
                        }
                    }
                }
            },
            1 => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'\'' {
                    state = 0;
                }
            }
            2 => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'"' {
                    state = 0;
                }
            }
            3 => {
                if b == b'\\' {
                    i += 1;
                } else if b == b'`' {
                    state = 0;
                }
            }
            _ => state = 0,
        }
        i += 1;
    }

    Ok(None)
}

fn parse_for_in_of_var(raw: &str) -> Result<String> {
    let mut cursor = Cursor::new(raw);
    cursor.skip_ws();
    let first = cursor
        .parse_identifier()
        .ok_or_else(|| Error::ScriptParse(format!("invalid for statement variable: {raw}")))?;

    let name = if matches!(first.as_str(), "let" | "const" | "var") {
        cursor.skip_ws();
        let name = cursor
            .parse_identifier()
            .ok_or_else(|| Error::ScriptParse(format!("invalid for statement variable: {raw}")))?;
        name
    } else {
        first
    };

    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "invalid for statement declaration: {raw}"
        )));
    }
    if !is_ident(&name) {
        return Err(Error::ScriptParse(format!(
            "invalid for statement variable: {raw}"
        )));
    }
    Ok(name)
}

fn parse_for_clause_stmt(src: &str) -> Result<Option<Box<Stmt>>> {
    let src = src.trim();
    if src.is_empty() {
        return Ok(None);
    }

    if let Some(parsed) = parse_var_decl(src)? {
        return Ok(Some(Box::new(parsed)));
    }

    if let Some(parsed) = parse_var_assign(src)? {
        return Ok(Some(Box::new(parsed)));
    }

    if let Some(parsed) = parse_for_update_stmt(src) {
        return Ok(Some(Box::new(parsed)));
    }

    let expr = parse_expr(src).map_err(|_| {
        Error::ScriptParse(format!("unsupported for-loop clause: {src}"))
    })?;
    Ok(Some(Box::new(Stmt::Expr(expr))))
}

fn parse_for_update_stmt(src: &str) -> Option<Stmt> {
    parse_update_stmt(src)
}

fn parse_update_stmt(stmt: &str) -> Option<Stmt> {
    let src = stmt.trim();

    if let Some(name) = src.strip_prefix("++") {
        let name = name.trim();
        if is_ident(name) {
            return Some(Stmt::VarAssign {
                name: name.to_string(),
                op: VarAssignOp::Add,
                expr: Expr::Number(1),
            });
        }
    }

    if let Some(name) = src.strip_prefix("--") {
        let name = name.trim();
        if is_ident(name) {
            return Some(Stmt::VarAssign {
                name: name.to_string(),
                op: VarAssignOp::Add,
                expr: Expr::Number(-1),
            });
        }
    }

    if let Some(name) = src.strip_suffix("++") {
        let name = name.trim();
        if is_ident(name) {
            return Some(Stmt::VarAssign {
                name: name.to_string(),
                op: VarAssignOp::Add,
                expr: Expr::Number(1),
            });
        }
    }

    if let Some(name) = src.strip_suffix("--") {
        let name = name.trim();
        if is_ident(name) {
            return Some(Stmt::VarAssign {
                name: name.to_string(),
                op: VarAssignOp::Add,
                expr: Expr::Number(-1),
            });
        }
    }

    None
}

fn split_top_level_statements(body: &str) -> Vec<String> {
    let bytes = body.as_bytes();
    let mut out = Vec::new();
    let mut start = 0;
    let mut i = 0;

    let mut paren = 0usize;
    let mut bracket = 0usize;
    let mut brace = 0usize;
    let mut brace_open_stack = Vec::new();

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
                b'{' => {
                    brace += 1;
                    brace_open_stack.push(i);
                }
                b'}' => {
                    brace = brace.saturating_sub(1);
                    let block_open = brace_open_stack.pop();
                    if paren == 0 && bracket == 0 && brace == 0 {
                        let tail = body.get(i + 1..).unwrap_or_default();
                        if should_split_after_closing_brace(body, block_open, tail) {
                            if let Some(part) = body.get(start..=i) {
                                out.push(part.to_string());
                            }
                            start = i + 1;
                        }
                    }
                }
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

fn should_split_after_closing_brace(body: &str, block_open: Option<usize>, tail: &str) -> bool {
    let tail = tail.trim_start();
    if tail.is_empty() {
        return false;
    }
    if is_keyword_prefix(tail, "else") {
        return false;
    }
    if is_keyword_prefix(tail, "while")
        && block_open.is_some_and(|open| is_do_block_prefix(body, open))
    {
        return false;
    }
    true
}

fn is_do_block_prefix(body: &str, block_open: usize) -> bool {
    let bytes = body.as_bytes();
    if block_open == 0 || block_open > bytes.len() {
        return false;
    }

    let mut j = block_open;
    while j > 0 && bytes[j - 1].is_ascii_whitespace() {
        j -= 1;
    }
    if j < 2 {
        return false;
    }
    if &bytes[j - 2..j] != b"do" {
        return false;
    }
    match bytes.get(j - 3) {
        Some(&b) => !is_ident_char(b),
        None => true,
    }
}

fn is_keyword_prefix(src: &str, keyword: &str) -> bool {
    let Some(rest) = src.strip_prefix(keyword) else {
        return false;
    };
    rest.is_empty() || !is_ident_char(*rest.as_bytes().first().unwrap_or(&b'\0'))
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

fn parse_var_assign(stmt: &str) -> Result<Option<Stmt>> {
    let stmt = stmt.trim();
    let Some((name, op_len, value_src)) = find_top_level_var_assignment(stmt) else {
        return Ok(None);
    };

    if name.is_empty() || !is_ident(&name) {
        return Ok(None);
    }

    let split_pos = stmt.len() - value_src.len();
    let op = match &stmt[split_pos - op_len..split_pos] {
        "=" => VarAssignOp::Assign,
        "+=" => VarAssignOp::Add,
        "-=" => VarAssignOp::Sub,
        "*=" => VarAssignOp::Mul,
        "/=" => VarAssignOp::Div,
        "**=" => VarAssignOp::Pow,
        "%=" => VarAssignOp::Mod,
        "|=" => VarAssignOp::BitOr,
        "^=" => VarAssignOp::BitXor,
        "&=" => VarAssignOp::BitAnd,
        "<<=" => VarAssignOp::ShiftLeft,
        ">>=" => VarAssignOp::ShiftRight,
        ">>>=" => VarAssignOp::UnsignedShiftRight,
        _ => {
            return Err(Error::ScriptParse(format!(
                "unsupported assignment operator: {stmt}"
            )))
        }
    };

    let expr = parse_expr(value_src)?;
    Ok(Some(Stmt::VarAssign {
        name: name.to_string(),
        op,
        expr,
    }))
}

fn find_top_level_var_assignment(stmt: &str) -> Option<(String, usize, &str)> {
    let (eq_pos, op_len) = find_top_level_assignment(stmt)?;
    let lhs = stmt[..eq_pos].trim();
    if lhs.is_empty() {
        return None;
    }

    Some((
        lhs.to_string(),
        op_len,
        stmt.get(eq_pos + op_len..).unwrap_or_default(),
    ))
}

fn parse_form_data_append_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let stmt = stmt.trim();
    let mut cursor = Cursor::new(stmt);
    cursor.skip_ws();

    let Some(target_var) = cursor.parse_identifier() else {
        return Ok(None);
    };

    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();

    let Some(method) = cursor.parse_identifier() else {
        return Ok(None);
    };
    if method != "append" {
        return Ok(None);
    }

    cursor.skip_ws();
    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.len() != 2 {
        return Ok(None);
    }

    let name = parse_expr(args[0].trim())?;
    let value = parse_expr(args[1].trim())?;

    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported FormData.append statement tail: {stmt}"
        )));
    }

    Ok(Some(Stmt::FormDataAppend {
        target_var,
        name,
        value,
    }))
}

fn parse_dom_method_call_stmt(stmt: &str) -> Result<Option<Stmt>> {
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

    let Some(method_name) = cursor.parse_identifier() else {
        return Ok(None);
    };
    let method = match method_name.as_str() {
        "focus" => DomMethod::Focus,
        "blur" => DomMethod::Blur,
        "click" => DomMethod::Click,
        "scrollIntoView" => DomMethod::ScrollIntoView,
        "submit" => DomMethod::Submit,
        "reset" => DomMethod::Reset,
        _ => return Ok(None),
    };

    cursor.skip_ws();
    let args = cursor.read_balanced_block(b'(', b')')?;
    if !args.trim().is_empty() {
        return Err(Error::ScriptParse(format!(
            "{} takes no arguments: {stmt}",
            method_name
        )));
    }

    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported {} statement tail: {stmt}",
            method_name
        )));
    }

    Ok(Some(Stmt::DomMethodCall { target, method }))
}

fn parse_dom_assignment(stmt: &str) -> Result<Option<Stmt>> {
    let Some((eq_pos, op_len)) = find_top_level_assignment(stmt) else {
        return Ok(None);
    };

    let lhs = stmt[..eq_pos].trim();
    let rhs = stmt[eq_pos + op_len..].trim();

    if lhs.is_empty() {
        return Ok(None);
    }

    if op_len != 1 {
        return Ok(None);
    }

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

    let source = match parse_element_target(&mut cursor) {
        Ok(target) => target,
        Err(_) => return Ok(None),
    };
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    let method = cursor
        .parse_identifier()
        .ok_or_else(|| Error::ScriptParse(format!("invalid forEach statement: {stmt}")))?;

    let (target, selector) = match method.as_str() {
        "forEach" => {
            let (target, selector) = match &source {
                DomQuery::BySelectorAll { selector } => (None, selector.clone()),
                DomQuery::QuerySelectorAll { target, selector } => {
                    (Some(target.as_ref().clone()), selector.clone())
                }
                _ => {
                    return Ok(None);
                }
            };
            cursor.skip_ws();
            (target, selector)
        }
        "querySelectorAll" => {
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
            (
                match source {
                    DomQuery::DocumentRoot => None,
                    _ => Some(source.clone()),
                },
                selector,
            )
        }
        _ => return Ok(None),
    };
    cursor.skip_ws();

    // For consistency with current test grammar, allow optional event callback without a semicolon.
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
        target,
        selector,
        item_var,
        index_var,
        body,
    }))
}

fn parse_array_for_each_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let stmt = stmt.trim();
    let mut cursor = Cursor::new(stmt);
    cursor.skip_ws();
    let Some(target) = cursor.parse_identifier() else {
        return Ok(None);
    };

    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("forEach") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.len() != 1 || args[0].trim().is_empty() {
        return Err(Error::ScriptParse(
            "forEach requires exactly one callback argument".into(),
        ));
    }
    let callback = parse_array_callback_arg(args[0], 3, "array callback parameters")?;

    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported forEach statement tail: {stmt}"
        )));
    }

    Ok(Some(Stmt::ArrayForEach { target, callback }))
}

fn parse_for_each_callback(src: &str) -> Result<(String, Option<String>, Vec<Stmt>)> {
    let mut cursor = Cursor::new(src.trim());
    cursor.skip_ws();

    let (item_var, index_var) = if cursor
        .src
        .get(cursor.i..)
        .is_some_and(|src| src.starts_with("function"))
        && !cursor
            .bytes()
            .get(cursor.i + "function".len())
            .is_some_and(|&b| is_ident_char(b))
    {
        cursor.consume_ascii("function");
        cursor.skip_ws();
        if !cursor.consume_byte(b'(') {
            let _ = cursor
                .parse_identifier()
                .ok_or_else(|| Error::ScriptParse("expected function name".into()))?;
            cursor.skip_ws();
            cursor.expect_byte(b'(')?;
        }
        let params_src = cursor.read_until_byte(b')')?;
        cursor.expect_byte(b')')?;
        let params = parse_callback_parameter_list(
            &params_src,
            2,
            "forEach callback must have one or two parameters",
        )?;
        let item_var = params.first().cloned().ok_or_else(|| {
            Error::ScriptParse(format!(
                "forEach callback must have one or two parameters: {src}"
            ))
        })?;
        let index_var = params.get(1).cloned();

        cursor.skip_ws();
    let body = parse_arrow_or_block_body(&mut cursor)?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported forEach callback tail: {src}"
        )));
        }

        return Ok((item_var, index_var, parse_block_statements(&body)?));
    } else if cursor.consume_byte(b'(') {
        let params_src = cursor.read_until_byte(b')')?;
        cursor.expect_byte(b')')?;
        let params = parse_callback_parameter_list(
            &params_src,
            2,
            "forEach callback must have one or two parameters",
        )?;
        let item_var = params.first().cloned().ok_or_else(|| {
            Error::ScriptParse(format!(
                "forEach callback must have one or two parameters: {src}"
            ))
        })?;
        let index_var = params.get(1).cloned();
        (item_var, index_var)
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
    let body = parse_arrow_or_block_body(&mut cursor)?;
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

    if method == "forEach" {
        cursor.skip_ws();
        let callback_src = cursor.read_balanced_block(b'(', b')')?;
        let (item_var, index_var, body) = parse_for_each_callback(&callback_src)?;

        cursor.skip_ws();
        cursor.consume_byte(b';');
        cursor.skip_ws();
        if !cursor.eof() {
            return Err(Error::ScriptParse(format!(
                "unsupported classList statement tail: {stmt}"
            )));
        }

        return Ok(Some(Stmt::ClassListForEach {
            target,
            item_var,
            index_var,
            body,
        }));
    }

    let method = match method.as_str() {
        "add" => ClassListMethod::Add,
        "remove" => ClassListMethod::Remove,
        "toggle" => ClassListMethod::Toggle,
        _ => return Ok(None),
    };

    cursor.skip_ws();
    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.is_empty() {
        return Err(Error::ScriptParse(format!(
            "invalid classList arguments: {stmt}"
        )));
    }

    let force = match method {
        ClassListMethod::Toggle => {
            if args.len() > 2 {
                return Err(Error::ScriptParse(format!(
                    "invalid classList arguments: {stmt}"
                )));
            }

            if args.len() == 2 {
                Some(parse_expr(args[1].trim())?)
            } else {
                None
            }
        }
        _ => None,
    };

    let class_names = match method {
        ClassListMethod::Toggle => vec![parse_string_literal_exact(args[0].trim())?],
        _ => args
            .iter()
            .map(|arg| parse_string_literal_exact(arg.trim()))
            .collect::<Result<Vec<_>>>()?,
    };

    if !matches!(method, ClassListMethod::Toggle) && class_names.is_empty() {
        return Err(Error::ScriptParse(format!(
            "classList add/remove requires at least one argument: {stmt}"
        )));
    }

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
        class_names,
        force,
    }))
}

fn parse_insert_adjacent_position(src: &str) -> Result<InsertAdjacentPosition> {
    let lowered = src.to_ascii_lowercase();
    match lowered.as_str() {
        "beforebegin" => Ok(InsertAdjacentPosition::BeforeBegin),
        "afterbegin" => Ok(InsertAdjacentPosition::AfterBegin),
        "beforeend" => Ok(InsertAdjacentPosition::BeforeEnd),
        "afterend" => Ok(InsertAdjacentPosition::AfterEnd),
        _ => Err(Error::ScriptParse(format!(
            "unsupported insertAdjacent position: {src}"
        ))),
    }
}

fn resolve_insert_adjacent_position(src: &str) -> Result<InsertAdjacentPosition> {
    let lowered = src.to_ascii_lowercase();
    match lowered.as_str() {
        "beforebegin" => Ok(InsertAdjacentPosition::BeforeBegin),
        "afterbegin" => Ok(InsertAdjacentPosition::AfterBegin),
        "beforeend" => Ok(InsertAdjacentPosition::BeforeEnd),
        "afterend" => Ok(InsertAdjacentPosition::AfterEnd),
        _ => Err(Error::ScriptRuntime(format!(
            "unsupported insertAdjacentHTML position: {src}"
        ))),
    }
}

fn parse_insert_adjacent_element_stmt(stmt: &str) -> Result<Option<Stmt>> {
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
    if !cursor.consume_ascii("insertAdjacentElement") {
        return Ok(None);
    }
    cursor.skip_ws();

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.len() != 2 {
        return Err(Error::ScriptParse(format!(
            "insertAdjacentElement requires 2 arguments: {stmt}"
        )));
    }

    let position = parse_insert_adjacent_position(&parse_string_literal_exact(args[0].trim())?)?;
    let node = parse_expr(args[1].trim())?;

    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported insertAdjacentElement statement tail: {stmt}"
        )));
    }

    Ok(Some(Stmt::InsertAdjacentElement {
        target,
        position,
        node,
    }))
}

fn parse_insert_adjacent_text_stmt(stmt: &str) -> Result<Option<Stmt>> {
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
    if !cursor.consume_ascii("insertAdjacentText") {
        return Ok(None);
    }
    cursor.skip_ws();

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.len() != 2 {
        return Err(Error::ScriptParse(format!(
            "insertAdjacentText requires 2 arguments: {stmt}"
        )));
    }

    let position = parse_insert_adjacent_position(&parse_string_literal_exact(args[0].trim())?)?;
    let text = parse_expr(args[1].trim())?;

    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported insertAdjacentText statement tail: {stmt}"
        )));
    }

    Ok(Some(Stmt::InsertAdjacentText {
        target,
        position,
        text,
    }))
}

fn parse_insert_adjacent_html_stmt(stmt: &str) -> Result<Option<Stmt>> {
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
    if !cursor.consume_ascii("insertAdjacentHTML") {
        return Ok(None);
    }
    cursor.skip_ws();

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.len() != 2 {
        return Err(Error::ScriptParse(format!(
            "insertAdjacentHTML requires 2 arguments: {stmt}"
        )));
    }

    let position = parse_expr(args[0].trim())?;
    let html = parse_expr(args[1].trim())?;

    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported insertAdjacentHTML statement tail: {stmt}"
        )));
    }

    Ok(Some(Stmt::InsertAdjacentHTML {
        target,
        position,
        html,
    }))
}

fn parse_set_timer_call(
    cursor: &mut Cursor<'_>,
    timer_name: &str,
) -> Result<Option<(TimerInvocation, Expr)>> {
    cursor.skip_ws();
    if !cursor.consume_ascii(timer_name) {
        return Ok(None);
    }
    cursor.skip_ws();

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.is_empty() {
        return Err(Error::ScriptParse(format!(
            "{timer_name} requires at least 1 argument"
        )));
    }

    let callback_arg = strip_js_comments(args[0]);
    let callback = parse_timer_callback(timer_name, callback_arg.as_str().trim())?;

    let delay_ms = if args.len() >= 2 {
        let delay_src = strip_js_comments(args[1]).trim().to_string();
        if delay_src.is_empty() {
            Expr::Number(0)
        } else {
            parse_expr(&delay_src)?
        }
    } else {
        Expr::Number(0)
    };

    let mut extra_args = Vec::new();
    for arg in args.iter().skip(2) {
        let arg_src = strip_js_comments(arg);
        if arg_src.trim().is_empty() {
            continue;
        }
        extra_args.push(parse_expr(arg_src.trim())?);
    }

    Ok(Some((TimerInvocation { callback, args: extra_args }, delay_ms)))
}

fn parse_set_timeout_call(cursor: &mut Cursor<'_>) -> Result<Option<(TimerInvocation, Expr)>> {
    parse_set_timer_call(cursor, "setTimeout")
}

fn parse_set_interval_call(cursor: &mut Cursor<'_>) -> Result<Option<(TimerInvocation, Expr)>> {
    parse_set_timer_call(cursor, "setInterval")
}

fn parse_set_timeout_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let stmt = stmt.trim();
    let mut cursor = Cursor::new(stmt);
    let Some((handler, delay_ms)) = parse_set_timeout_call(&mut cursor)? else {
        return Ok(None);
    };

    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported setTimeout statement tail: {stmt}"
        )));
    }

    Ok(Some(Stmt::SetTimeout { handler, delay_ms }))
}

fn parse_set_interval_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let stmt = stmt.trim();
    let mut cursor = Cursor::new(stmt);
    let Some((handler, delay_ms)) = parse_set_interval_call(&mut cursor)? else {
        return Ok(None);
    };

    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported setInterval statement tail: {stmt}"
        )));
    }

    Ok(Some(Stmt::SetInterval { handler, delay_ms }))
}

fn parse_clear_timeout_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let stmt = stmt.trim();
    let mut cursor = Cursor::new(stmt);
    cursor.skip_ws();
    let method = if cursor.consume_ascii("clearTimeout") {
        "clearTimeout"
    } else if cursor.consume_ascii("clearInterval") {
        "clearInterval"
    } else {
        return Ok(None);
    };
    cursor.skip_ws();

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.len() != 1 {
        return Err(Error::ScriptParse(format!(
            "{method} requires 1 argument: {stmt}"
        )));
    }
    let timer_id = parse_expr(args[0].trim())?;

    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported {method} statement tail: {stmt}"
        )));
    }

    Ok(Some(Stmt::ClearTimeout { timer_id }))
}

fn parse_node_tree_mutation_stmt(stmt: &str) -> Result<Option<Stmt>> {
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
    let method = match method.as_str() {
        "after" => NodeTreeMethod::After,
        "append" => NodeTreeMethod::Append,
        "appendChild" => NodeTreeMethod::AppendChild,
        "before" => NodeTreeMethod::Before,
        "replaceWith" => NodeTreeMethod::ReplaceWith,
        "prepend" => NodeTreeMethod::Prepend,
        "removeChild" => NodeTreeMethod::RemoveChild,
        "insertBefore" => NodeTreeMethod::InsertBefore,
        _ => return Ok(None),
    };
    cursor.skip_ws();

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    let (method_name, expected_args) = match method {
        NodeTreeMethod::After => ("after", 1),
        NodeTreeMethod::Append => ("append", 1),
        NodeTreeMethod::AppendChild => ("appendChild", 1),
        NodeTreeMethod::Before => ("before", 1),
        NodeTreeMethod::ReplaceWith => ("replaceWith", 1),
        NodeTreeMethod::Prepend => ("prepend", 1),
        NodeTreeMethod::RemoveChild => ("removeChild", 1),
        NodeTreeMethod::InsertBefore => ("insertBefore", 2),
    };
    if args.len() != expected_args {
        return Err(Error::ScriptParse(format!(
            "{} requires {} argument{}: {}",
            method_name,
            expected_args,
            if expected_args == 1 { "" } else { "s" },
            stmt
        )));
    }
    let child = parse_expr(args[0].trim())?;
    let reference = if method == NodeTreeMethod::InsertBefore {
        Some(parse_expr(args[1].trim())?)
    } else {
        None
    };

    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported node tree mutation statement tail: {stmt}"
        )));
    }

    Ok(Some(Stmt::NodeTreeMutation {
        target,
        method,
        child,
        reference,
    }))
}

fn parse_node_remove_stmt(stmt: &str) -> Result<Option<Stmt>> {
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
    if method != "remove" {
        return Ok(None);
    }
    cursor.skip_ws();
    cursor.expect_byte(b'(')?;
    cursor.skip_ws();
    cursor.expect_byte(b')')?;
    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported remove() statement tail: {stmt}"
        )));
    }

    Ok(Some(Stmt::NodeRemove { target }))
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
    let (params, body) = parse_callback(&mut cursor, 1, "callback parameters")?;

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
        params,
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

    if let Some(handler_expr) = parse_function_expr(src)? {
        return Ok(handler_expr);
    }

    parse_ternary_expr(src)
}

fn strip_js_comments(src: &str) -> String {
    enum State {
        Normal,
        Single,
        Double,
        Template,
    }

    let bytes = src.as_bytes();
    let mut state = State::Normal;
    let mut i = 0usize;
    let mut out: Vec<u8> = Vec::with_capacity(src.len());

    while i < bytes.len() {
        let b = bytes[i];
        match state {
            State::Normal => {
                if b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                    i += 2;
                    while i < bytes.len() && bytes[i] != b'\n' {
                        i += 1;
                    }
                    if i < bytes.len() {
                        out.push(b'\n');
                        i += 1;
                    }
                    continue;
                }
                if b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
                    i += 2;
                    while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                        i += 1;
                    }
                    if i + 1 < bytes.len() {
                        i += 2;
                    } else {
                        i = bytes.len();
                    }
                    continue;
                }

                match b {
                    b'\'' => {
                        state = State::Single;
                        out.push(b);
                        i += 1;
                    }
                    b'"' => {
                        state = State::Double;
                        out.push(b);
                        i += 1;
                    }
                    b'`' => {
                        state = State::Template;
                        out.push(b);
                        i += 1;
                    }
                    _ => {
                        out.push(b);
                        i += 1;
                    }
                }
            }
            State::Single => {
                if b == b'\\' {
                    out.push(b);
                    if i + 1 < bytes.len() {
                        out.push(bytes[i + 1]);
                        i += 2;
                    } else {
                        i += 1;
                    }
                    continue;
                }
                out.push(b);
                if b == b'\'' {
                    state = State::Normal;
                }
                i += 1;
            }
            State::Double => {
                if b == b'\\' {
                    out.push(b);
                    if i + 1 < bytes.len() {
                        out.push(bytes[i + 1]);
                        i += 2;
                    } else {
                        i += 1;
                    }
                    continue;
                }
                out.push(b);
                if b == b'"' {
                    state = State::Normal;
                }
                i += 1;
            }
            State::Template => {
                if b == b'\\' {
                    out.push(b);
                    if i + 1 < bytes.len() {
                        out.push(bytes[i + 1]);
                        i += 2;
                    } else {
                        i += 1;
                    }
                    continue;
                }
                out.push(b);
                if b == b'`' {
                    state = State::Normal;
                }
                i += 1;
            }
        }
    }

    String::from_utf8(out).unwrap_or_else(|_| src.to_string())
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
        return parse_bitwise_or_expr(src);
    }
    fold_binary(parts, ops, parse_bitwise_or_expr, |op| match op {
        "&&" => BinaryOp::And,
        _ => unreachable!(),
    })
}

fn parse_bitwise_or_expr(src: &str) -> Result<Expr> {
    let src = strip_outer_parens(src.trim());
    let (parts, ops) = split_top_level_by_ops(src, &["|"]);
    if ops.is_empty() {
        return parse_bitwise_xor_expr(src);
    }
    fold_binary(parts, ops, parse_bitwise_xor_expr, |op| match op {
        "|" => BinaryOp::BitOr,
        _ => unreachable!(),
    })
}

fn parse_bitwise_xor_expr(src: &str) -> Result<Expr> {
    let src = strip_outer_parens(src.trim());
    let (parts, ops) = split_top_level_by_ops(src, &["^"]);
    if ops.is_empty() {
        return parse_bitwise_and_expr(src);
    }
    fold_binary(parts, ops, parse_bitwise_and_expr, |op| match op {
        "^" => BinaryOp::BitXor,
        _ => unreachable!(),
    })
}

fn parse_bitwise_and_expr(src: &str) -> Result<Expr> {
    let src = strip_outer_parens(src.trim());
    let (parts, ops) = split_top_level_by_ops(src, &["&"]);
    if ops.is_empty() {
        return parse_equality_expr(src);
    }
    fold_binary(parts, ops, parse_equality_expr, |op| match op {
        "&" => BinaryOp::BitAnd,
        _ => unreachable!(),
    })
}

fn parse_equality_expr(src: &str) -> Result<Expr> {
    let src = strip_outer_parens(src.trim());
    let (parts, ops) = split_top_level_by_ops(src, &["!==", "===", "!=", "=="]);
    if ops.is_empty() {
        return parse_relational_expr(src);
    }
    fold_binary(parts, ops, parse_relational_expr, |op| match op {
        "===" => BinaryOp::StrictEq,
        "!==" => BinaryOp::StrictNe,
        "==" => BinaryOp::StrictEq,
        "!=" => BinaryOp::StrictNe,
        _ => unreachable!(),
    })
}

fn parse_relational_expr(src: &str) -> Result<Expr> {
    let src = strip_outer_parens(src.trim());
    let (parts, ops) = split_top_level_by_ops(src, &["<=", ">=", "<", ">", "instanceof", "in"]);
    if ops.is_empty() {
        return parse_shift_expr(src);
    }
    fold_binary(parts, ops, parse_shift_expr, |op| match op {
        "<" => BinaryOp::Lt,
        ">" => BinaryOp::Gt,
        "<=" => BinaryOp::Le,
        ">=" => BinaryOp::Ge,
        "instanceof" => BinaryOp::InstanceOf,
        "in" => BinaryOp::In,
        _ => unreachable!(),
    })
}

fn parse_shift_expr(src: &str) -> Result<Expr> {
    let src = strip_outer_parens(src.trim());
    let (parts, ops) = split_top_level_by_ops(src, &[">>>", "<<", ">>"]);
    if ops.is_empty() {
        return parse_add_expr(src);
    }
    fold_binary(parts, ops, parse_add_expr, |op| match op {
        ">>>" => BinaryOp::UnsignedShiftRight,
        "<<" => BinaryOp::ShiftLeft,
        ">>" => BinaryOp::ShiftRight,
        _ => unreachable!(),
    })
}

fn parse_add_expr(src: &str) -> Result<Expr> {
    let src = strip_outer_parens(src.trim());
    let (parts, ops) = split_top_level_add_sub(src);
    if ops.is_empty() {
        return parse_mul_expr(src);
    }

    if parts.iter().any(|part| part.trim().is_empty()) {
        return Err(Error::ScriptParse(format!(
            "invalid additive expression: {src}"
        )));
    }

    let mut expr = parse_mul_expr(parts[0].trim())?;
    for (idx, op) in ops.iter().enumerate() {
        let rhs = parse_expr(parts[idx + 1].trim())?;
        if *op == '+' {
            expr = append_concat_expr(expr, rhs);
        } else {
            expr = Expr::Binary {
                left: Box::new(expr),
                op: BinaryOp::Sub,
                right: Box::new(rhs),
            };
        }
    }

    Ok(expr)
}

fn append_concat_expr(lhs: Expr, rhs: Expr) -> Expr {
    match lhs {
        Expr::Add(mut parts) => {
            parts.push(rhs);
            Expr::Add(parts)
        }
        other => Expr::Add(vec![other, rhs]),
    }
}

fn split_top_level_add_sub(src: &str) -> (Vec<&str>, Vec<char>) {
    let bytes = src.as_bytes();
    let mut parts = Vec::new();
    let mut ops = Vec::new();
    let mut start = 0usize;

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

    let mut i = 0usize;
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
                b'+' | b'-' if paren == 0 && bracket == 0 && brace == 0 => {
                    if is_add_sub_binary_operator(bytes, i) {
                        if let Some(part) = src.get(start..i) {
                            parts.push(part);
                        }
                        ops.push(b as char);
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

    if let Some(part) = src.get(start..) {
        parts.push(part);
    }

    (parts, ops)
}

fn is_add_sub_binary_operator(bytes: &[u8], idx: usize) -> bool {
    if idx >= bytes.len() {
        return false;
    }
    let mut left = idx;
    while left > 0 && bytes[left - 1].is_ascii_whitespace() {
        left -= 1;
    }
    if left == 0 {
        return false;
    }
    let prev = bytes[left - 1];
    !matches!(
        prev,
        b'(' | b'['
            | b'{'
            | b','
            | b'?'
            | b':'
            | b'='
            | b'!'
            | b'<'
            | b'>'
            | b'&'
            | b'|'
            | b'+'
            | b'-'
            | b'*'
            | b'/'
            | b'%'
    )
}

fn parse_mul_expr(src: &str) -> Result<Expr> {
    let src = strip_outer_parens(src.trim());
    let bytes = src.as_bytes();
    let mut parts = Vec::new();
    let mut ops: Vec<u8> = Vec::new();
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
                b'/' | b'%' => {
                    if paren == 0 && bracket == 0 && brace == 0 {
                        if let Some(part) = src.get(start..i) {
                            parts.push(part);
                            ops.push(b);
                            start = i + 1;
                        }
                    }
                }
                b'*' if paren == 0 && bracket == 0 && brace == 0 => {
                    if i + 1 < bytes.len() && bytes[i + 1] == b'*' {
                        i += 1;
                    } else if let Some(part) = src.get(start..i) {
                        parts.push(part);
                        ops.push(b);
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

    if let Some(last) = src.get(start..) {
        parts.push(last);
    }

    if ops.is_empty() {
        return parse_pow_expr(src);
    }

    let mut expr = parse_pow_expr(parts[0].trim())?;
    for (idx, op) in ops.iter().enumerate() {
        let rhs = parse_pow_expr(parts[idx + 1].trim())?;
        let op = match op {
            b'/' => BinaryOp::Div,
            b'%' => BinaryOp::Mod,
            _ => BinaryOp::Mul,
        };
        expr = Expr::Binary {
            left: Box::new(expr),
            op,
            right: Box::new(rhs),
        };
    }
    Ok(expr)
}

fn parse_pow_expr(src: &str) -> Result<Expr> {
    let src = strip_outer_parens(src.trim());
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
                b'*' if paren == 0 && bracket == 0 && brace == 0 => {
                    if i + 1 < bytes.len() && bytes[i + 1] == b'*' {
                        let left = parse_expr(src[..i].trim())?;
                        let right = parse_pow_expr(src[i + 2..].trim())?;
                        return Ok(Expr::Binary {
                            left: Box::new(left),
                            op: BinaryOp::Pow,
                            right: Box::new(right),
                        });
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

    parse_unary_expr(src)
}

fn parse_unary_expr(src: &str) -> Result<Expr> {
    let src = strip_outer_parens(src.trim());
    if let Some(rest) = strip_keyword_operator(src, "typeof") {
        let inner = parse_unary_expr(rest)?;
        return Ok(Expr::TypeOf(Box::new(inner)));
    }
    if let Some(rest) = strip_keyword_operator(src, "void") {
        let inner = parse_unary_expr(rest)?;
        return Ok(Expr::Void(Box::new(inner)));
    }
    if let Some(rest) = strip_keyword_operator(src, "delete") {
        let inner = parse_unary_expr(rest)?;
        return Ok(Expr::Delete(Box::new(inner)));
    }
    if let Some(rest) = src.strip_prefix('+') {
        let inner = parse_unary_expr(rest.trim())?;
        return Ok(Expr::Pos(Box::new(inner)));
    }
    if let Some(rest) = src.strip_prefix('-') {
        let inner = parse_unary_expr(rest.trim())?;
        return Ok(Expr::Neg(Box::new(inner)));
    }
    if let Some(rest) = src.strip_prefix('!') {
        let inner = parse_unary_expr(rest.trim())?;
        return Ok(Expr::Not(Box::new(inner)));
    }
    if let Some(rest) = src.strip_prefix('~') {
        let inner = parse_unary_expr(rest.trim())?;
        return Ok(Expr::BitNot(Box::new(inner)));
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
    if src == "null" {
        return Ok(Expr::Null);
    }
    if src == "undefined" {
        return Ok(Expr::Undefined);
    }
    if src == "NaN" {
        return Ok(Expr::Float(f64::NAN));
    }
    if src == "Infinity" {
        return Ok(Expr::Float(f64::INFINITY));
    }
    if let Some(numeric) = parse_numeric_literal(src)? {
        return Ok(numeric);
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

    if let Some(expr) = parse_new_date_expr(src)? {
        return Ok(expr);
    }

    if parse_date_now_expr(src)? {
        return Ok(Expr::DateNow);
    }

    if let Some(value) = parse_date_parse_expr(src)? {
        return Ok(Expr::DateParse(Box::new(value)));
    }

    if let Some(args) = parse_date_utc_expr(src)? {
        return Ok(Expr::DateUtc { args });
    }

    if parse_math_random_expr(src)? {
        return Ok(Expr::MathRandom);
    }

    if let Some(value) = parse_encode_uri_component_expr(src)? {
        return Ok(Expr::EncodeUriComponent(Box::new(value)));
    }

    if let Some(value) = parse_encode_uri_expr(src)? {
        return Ok(Expr::EncodeUri(Box::new(value)));
    }

    if let Some(value) = parse_decode_uri_component_expr(src)? {
        return Ok(Expr::DecodeUriComponent(Box::new(value)));
    }

    if let Some(value) = parse_decode_uri_expr(src)? {
        return Ok(Expr::DecodeUri(Box::new(value)));
    }

    if let Some(value) = parse_escape_expr(src)? {
        return Ok(Expr::Escape(Box::new(value)));
    }

    if let Some(value) = parse_unescape_expr(src)? {
        return Ok(Expr::Unescape(Box::new(value)));
    }

    if let Some(value) = parse_is_nan_expr(src)? {
        return Ok(Expr::IsNaN(Box::new(value)));
    }

    if let Some(value) = parse_is_finite_expr(src)? {
        return Ok(Expr::IsFinite(Box::new(value)));
    }

    if let Some(value) = parse_atob_expr(src)? {
        return Ok(Expr::Atob(Box::new(value)));
    }

    if let Some(value) = parse_btoa_expr(src)? {
        return Ok(Expr::Btoa(Box::new(value)));
    }

    if let Some((value, radix)) = parse_parse_int_expr(src)? {
        return Ok(Expr::ParseInt {
            value: Box::new(value),
            radix: radix.map(Box::new),
        });
    }

    if let Some(value) = parse_parse_float_expr(src)? {
        return Ok(Expr::ParseFloat(Box::new(value)));
    }

    if let Some(value) = parse_fetch_expr(src)? {
        return Ok(Expr::Fetch(Box::new(value)));
    }

    if let Some(value) = parse_alert_expr(src)? {
        return Ok(Expr::Alert(Box::new(value)));
    }

    if let Some(value) = parse_confirm_expr(src)? {
        return Ok(Expr::Confirm(Box::new(value)));
    }

    if let Some((message, default)) = parse_prompt_expr(src)? {
        return Ok(Expr::Prompt {
            message: Box::new(message),
            default: default.map(Box::new),
        });
    }

    if let Some(values) = parse_array_literal_expr(src)? {
        return Ok(Expr::ArrayLiteral(values));
    }

    if let Some(value) = parse_array_is_array_expr(src)? {
        return Ok(Expr::ArrayIsArray(Box::new(value)));
    }

    if let Some(expr) = parse_array_access_expr(src)? {
        return Ok(expr);
    }

    if let Some(expr) = parse_string_method_expr(src)? {
        return Ok(expr);
    }

    if let Some(expr) = parse_date_method_expr(src)? {
        return Ok(expr);
    }

    if let Some(tag_name) = parse_document_create_element_expr(src)? {
        return Ok(Expr::CreateElement(tag_name));
    }

    if let Some(text) = parse_document_create_text_node_expr(src)? {
        return Ok(Expr::CreateTextNode(text));
    }

    if let Some(handler_expr) = parse_function_expr(src)? {
        return Ok(handler_expr);
    }

    if let Some((handler, delay_ms)) = parse_set_timeout_expr(src)? {
        return Ok(Expr::SetTimeout {
            handler,
            delay_ms: Box::new(delay_ms),
        });
    }

    if let Some((handler, delay_ms)) = parse_set_interval_expr(src)? {
        return Ok(Expr::SetInterval {
            handler,
            delay_ms: Box::new(delay_ms),
        });
    }

    if let Some(handler) = parse_queue_microtask_expr(src)? {
        return Ok(Expr::QueueMicrotask { handler });
    }

    if let Some(callback) = parse_promise_then_expr(src)? {
        return Ok(Expr::PromiseThen { callback });
    }

    if let Some((target, class_name)) = parse_class_list_contains_expr(src)? {
        return Ok(Expr::ClassListContains { target, class_name });
    }

    if let Some(target) = parse_query_selector_all_length_expr(src)? {
        return Ok(Expr::QuerySelectorAllLength { target });
    }

    if let Some(form) = parse_form_elements_length_expr(src)? {
        return Ok(Expr::FormElementsLength { form });
    }

    if let Some((source, name)) = parse_form_data_get_all_length_expr(src)? {
        return Ok(Expr::FormDataGetAllLength { source, name });
    }

    if let Some((source, name)) = parse_form_data_get_expr(src)? {
        return Ok(Expr::FormDataGet { source, name });
    }

    if let Some((source, name)) = parse_form_data_has_expr(src)? {
        return Ok(Expr::FormDataHas { source, name });
    }

    if let Some(form) = parse_new_form_data_expr(src)? {
        return Ok(Expr::FormDataNew { form });
    }

    if let Some((target, name)) = parse_get_attribute_expr(src)? {
        return Ok(Expr::DomGetAttribute { target, name });
    }

    if let Some((target, name)) = parse_has_attribute_expr(src)? {
        return Ok(Expr::DomHasAttribute { target, name });
    }

    if let Some((target, selector)) = parse_dom_matches_expr(src)? {
        return Ok(Expr::DomMatches { target, selector });
    }

    if let Some((target, selector)) = parse_dom_closest_expr(src)? {
        return Ok(Expr::DomClosest { target, selector });
    }

    if let Some((target, property)) = parse_dom_computed_style_property_expr(src)? {
        return Ok(Expr::DomComputedStyleProperty { target, property });
    }

    if let Some((event_var, prop)) = parse_event_property_expr(src)? {
        return Ok(Expr::EventProp { event_var, prop });
    }

    if let Some((target, prop)) = parse_dom_access(src)? {
        return Ok(Expr::DomRead { target, prop });
    }

    if let Some(target) = parse_element_ref_expr(src)? {
        return Ok(Expr::DomRef(target));
    }

    if is_ident(src) {
        return Ok(Expr::Var(src.to_string()));
    }

    Err(Error::ScriptParse(format!("unsupported expression: {src}")))
}

fn parse_numeric_literal(src: &str) -> Result<Option<Expr>> {
    if src.is_empty() {
        return Ok(None);
    }

    if let Some(value) = parse_prefixed_integer_literal(src, "0x", 16)? {
        return Ok(Some(value));
    }
    if let Some(value) = parse_prefixed_integer_literal(src, "0o", 8)? {
        return Ok(Some(value));
    }
    if let Some(value) = parse_prefixed_integer_literal(src, "0b", 2)? {
        return Ok(Some(value));
    }

    if src.as_bytes().iter().any(|b| matches!(b, b'e' | b'E')) {
        if !matches!(src.as_bytes().first(), Some(b) if b.is_ascii_digit() || *b == b'.') {
            return Ok(None);
        }
        let n: f64 = src
            .parse()
            .map_err(|_| Error::ScriptParse(format!("invalid numeric literal: {src}")))?;
        if !n.is_finite() {
            return Err(Error::ScriptParse(format!("invalid numeric literal: {src}")));
        }
        return Ok(Some(Expr::Float(n)));
    }

    if src.as_bytes().iter().all(|b| b.is_ascii_digit()) {
        let n: i64 = src
            .parse()
            .map_err(|_| Error::ScriptParse(format!("invalid numeric literal: {src}")))?;
        return Ok(Some(Expr::Number(n)));
    }

    let mut dot_count = 0usize;
    for b in src.as_bytes() {
        if *b == b'.' {
            dot_count += 1;
        } else if !b.is_ascii_digit() {
            return Ok(None);
        }
    }

    if dot_count != 1 {
        return Ok(None);
    }
    if src.starts_with('.') || src.ends_with('.') {
        return Ok(None);
    }

    let n: f64 = src
        .parse()
        .map_err(|_| Error::ScriptParse(format!("invalid numeric literal: {src}")))?;
    if !n.is_finite() {
        return Err(Error::ScriptParse(format!(
            "invalid numeric literal: {src}"
        )));
    }
    Ok(Some(Expr::Float(n)))
}

fn parse_prefixed_integer_literal(src: &str, prefix: &str, radix: u32) -> Result<Option<Expr>> {
    let src = src.to_ascii_lowercase();
    if !src.starts_with(prefix) {
        return Ok(None);
    }

    let digits = &src[prefix.len()..];
    if digits.is_empty() {
        return Err(Error::ScriptParse(format!("invalid numeric literal: {src}")));
        }

    let n = i64::from_str_radix(digits, radix)
        .map_err(|_| Error::ScriptParse(format!("invalid numeric literal: {src}")))?;
    Ok(Some(Expr::Number(n)))
}

fn strip_keyword_operator<'a>(src: &'a str, keyword: &str) -> Option<&'a str> {
    if !src.starts_with(keyword) {
        return None;
    }

    let after = &src[keyword.len()..];
    if after.is_empty() || !is_ident_char(after.as_bytes()[0]) {
        return Some(after.trim_start());
    }

    None
}

fn parse_element_ref_expr(src: &str) -> Result<Option<DomQuery>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    let target = match parse_element_target(&mut cursor) {
        Ok(target) => target,
        Err(_) => return Ok(None),
    };
    cursor.skip_ws();
    if cursor.eof() && matches!(target, DomQuery::Var(_)) {
        return Ok(None);
    }
    Ok(Some(target))
}

fn parse_document_create_element_expr(src: &str) -> Result<Option<String>> {
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
    if !cursor.consume_ascii("createElement") {
        return Ok(None);
    }
    cursor.skip_ws();
    cursor.expect_byte(b'(')?;
    cursor.skip_ws();
    let tag_name = cursor.parse_string_literal()?;
    cursor.skip_ws();
    cursor.expect_byte(b')')?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(tag_name.to_ascii_lowercase()))
}

fn parse_document_create_text_node_expr(src: &str) -> Result<Option<String>> {
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
    if !cursor.consume_ascii("createTextNode") {
        return Ok(None);
    }
    cursor.skip_ws();
    cursor.expect_byte(b'(')?;
    cursor.skip_ws();
    let text = cursor.parse_string_literal()?;
    cursor.skip_ws();
    cursor.expect_byte(b')')?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(text))
}

fn parse_new_date_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    if !cursor.consume_ascii("new") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("Date") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    let args: Vec<String> = if cursor.peek() == Some(b'(') {
        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args.into_iter().map(|arg| arg.to_string()).collect()
        }
    } else {
        Vec::new()
    };

    if args.len() > 1 {
        return Err(Error::ScriptParse(
            "new Date supports zero or one argument".into(),
        ));
    }

    let value = if args.len() == 1 {
        if args[0].trim().is_empty() {
            return Err(Error::ScriptParse(
                "new Date argument cannot be empty".into(),
            ));
        }
        Some(Box::new(parse_expr(args[0].trim())?))
    } else {
        None
    };

    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    Ok(Some(Expr::DateNew { value }))
}

fn parse_date_now_expr(src: &str) -> Result<bool> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(false);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("Date") {
        return Ok(false);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(false);
        }
    }
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(false);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("now") {
        return Ok(false);
    }
    cursor.skip_ws();
    cursor.expect_byte(b'(')?;
    cursor.skip_ws();
    cursor.expect_byte(b')')?;
    cursor.skip_ws();
    Ok(cursor.eof())
}

fn parse_date_static_args_expr(src: &str, method: &str) -> Result<Option<Vec<String>>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("Date") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii(method) {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',')
        .into_iter()
        .map(|arg| arg.to_string())
        .collect::<Vec<_>>();
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(args))
}

fn parse_date_parse_expr(src: &str) -> Result<Option<Expr>> {
    let Some(args) = parse_date_static_args_expr(src, "parse")? else {
        return Ok(None);
    };
    if args.len() != 1 || args[0].trim().is_empty() {
        return Err(Error::ScriptParse(
            "Date.parse requires exactly one argument".into(),
        ));
    }
    Ok(Some(parse_expr(args[0].trim())?))
}

fn parse_date_utc_expr(src: &str) -> Result<Option<Vec<Expr>>> {
    let Some(args) = parse_date_static_args_expr(src, "UTC")? else {
        return Ok(None);
    };

    if args.len() < 2 || args.len() > 6 {
        return Err(Error::ScriptParse(
            "Date.UTC requires between 2 and 6 arguments".into(),
        ));
    }

    let mut out = Vec::with_capacity(args.len());
    for arg in args {
        if arg.trim().is_empty() {
            return Err(Error::ScriptParse(
                "Date.UTC argument cannot be empty".into(),
            ));
        }
        out.push(parse_expr(arg.trim())?);
    }
    Ok(Some(out))
}

fn parse_math_random_expr(src: &str) -> Result<bool> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    if !cursor.consume_ascii("Math") {
        return Ok(false);
    }
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(false);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("random") {
        return Ok(false);
    }
    cursor.skip_ws();
    cursor.expect_byte(b'(')?;
    cursor.skip_ws();
    cursor.expect_byte(b')')?;
    cursor.skip_ws();
    Ok(cursor.eof())
}

fn parse_is_nan_expr(src: &str) -> Result<Option<Expr>> {
    parse_global_single_arg_expr(src, "isNaN", "isNaN requires exactly one argument")
}

fn parse_encode_uri_expr(src: &str) -> Result<Option<Expr>> {
    parse_global_single_arg_expr(
        src,
        "encodeURI",
        "encodeURI requires exactly one argument",
    )
}

fn parse_encode_uri_component_expr(src: &str) -> Result<Option<Expr>> {
    parse_global_single_arg_expr(
        src,
        "encodeURIComponent",
        "encodeURIComponent requires exactly one argument",
    )
}

fn parse_decode_uri_expr(src: &str) -> Result<Option<Expr>> {
    parse_global_single_arg_expr(
        src,
        "decodeURI",
        "decodeURI requires exactly one argument",
    )
}

fn parse_decode_uri_component_expr(src: &str) -> Result<Option<Expr>> {
    parse_global_single_arg_expr(
        src,
        "decodeURIComponent",
        "decodeURIComponent requires exactly one argument",
    )
}

fn parse_escape_expr(src: &str) -> Result<Option<Expr>> {
    parse_global_single_arg_expr(src, "escape", "escape requires exactly one argument")
}

fn parse_unescape_expr(src: &str) -> Result<Option<Expr>> {
    parse_global_single_arg_expr(src, "unescape", "unescape requires exactly one argument")
}

fn parse_is_finite_expr(src: &str) -> Result<Option<Expr>> {
    parse_global_single_arg_expr(src, "isFinite", "isFinite requires exactly one argument")
}

fn parse_atob_expr(src: &str) -> Result<Option<Expr>> {
    parse_global_single_arg_expr(src, "atob", "atob requires exactly one argument")
}

fn parse_btoa_expr(src: &str) -> Result<Option<Expr>> {
    parse_global_single_arg_expr(src, "btoa", "btoa requires exactly one argument")
}

fn parse_global_single_arg_expr(src: &str, function_name: &str, arg_error: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii(function_name) {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.len() != 1 || args[0].trim().is_empty() {
        return Err(Error::ScriptParse(arg_error.into()));
    }

    let value = parse_expr(args[0].trim())?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(value))
}

fn parse_parse_int_expr(src: &str) -> Result<Option<(Expr, Option<Expr>)>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("parseInt") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
        return Err(Error::ScriptParse(
            "parseInt requires one or two arguments".into(),
        ));
    }
    if args.len() == 2 && args[1].trim().is_empty() {
        return Err(Error::ScriptParse(
            "parseInt radix argument cannot be empty".into(),
        ));
    }

    let value = parse_expr(args[0].trim())?;
    let radix = if args.len() == 2 {
        Some(parse_expr(args[1].trim())?)
    } else {
        None
    };

    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some((value, radix)))
}

fn parse_parse_float_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("parseFloat") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.len() != 1 || args[0].trim().is_empty() {
        return Err(Error::ScriptParse(
            "parseFloat requires exactly one argument".into(),
        ));
    }

    let value = parse_expr(args[0].trim())?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(value))
}

fn parse_fetch_expr(src: &str) -> Result<Option<Expr>> {
    parse_global_single_arg_expr(src, "fetch", "fetch requires exactly one argument")
}

fn parse_alert_expr(src: &str) -> Result<Option<Expr>> {
    parse_global_single_arg_expr(src, "alert", "alert requires exactly one argument")
}

fn parse_confirm_expr(src: &str) -> Result<Option<Expr>> {
    parse_global_single_arg_expr(src, "confirm", "confirm requires exactly one argument")
}

fn parse_prompt_expr(src: &str) -> Result<Option<(Expr, Option<Expr>)>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("prompt") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
        return Err(Error::ScriptParse(
            "prompt requires one or two arguments".into(),
        ));
    }
    if args.len() == 2 && args[1].trim().is_empty() {
        return Err(Error::ScriptParse(
            "prompt default argument cannot be empty".into(),
        ));
    }

    let message = parse_expr(args[0].trim())?;
    let default = if args.len() == 2 {
        Some(parse_expr(args[1].trim())?)
    } else {
        None
    };

    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some((message, default)))
}

fn parse_array_literal_expr(src: &str) -> Result<Option<Vec<Expr>>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    if cursor.peek() != Some(b'[') {
        return Ok(None);
    }

    let items_src = cursor.read_balanced_block(b'[', b']')?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    let items = split_top_level_by_char(&items_src, b',');
    if items.len() == 1 && items[0].trim().is_empty() {
        return Ok(Some(Vec::new()));
    }

    let mut out = Vec::with_capacity(items.len());
    for item in items {
        let item = item.trim();
        if item.is_empty() {
            return Err(Error::ScriptParse(
                "array literal does not support empty elements".into(),
            ));
        }
        out.push(parse_expr(item)?);
    }
    Ok(Some(out))
}

fn parse_array_is_array_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("Array") {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("isArray") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();
    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.len() != 1 || args[0].trim().is_empty() {
        return Err(Error::ScriptParse(
            "Array.isArray requires exactly one argument".into(),
        ));
    }

    let value = parse_expr(args[0].trim())?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(value))
}

fn parse_array_access_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    let Some(target) = cursor.parse_identifier() else {
        return Ok(None);
    };
    cursor.skip_ws();

    if cursor.consume_byte(b'[') {
        cursor.skip_ws();
        let index_src = cursor.read_until_byte(b']')?;
        cursor.skip_ws();
        cursor.expect_byte(b']')?;
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        if index_src.trim().is_empty() {
            return Err(Error::ScriptParse("array index cannot be empty".into()));
        }
        let index = parse_expr(index_src.trim())?;
        return Ok(Some(Expr::ArrayIndex {
            target,
            index: Box::new(index),
        }));
    }

    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    let Some(method) = cursor.parse_identifier() else {
        return Ok(None);
    };
    cursor.skip_ws();

    if method == "length" {
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::ArrayLength(target)));
    }

    if cursor.peek() != Some(b'(') {
        return Ok(None);
    }

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let raw_args = split_top_level_by_char(&args_src, b',');
    let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
        Vec::new()
    } else {
        raw_args
    };

    let expr = match method.as_str() {
        "push" => {
            let mut parsed = Vec::with_capacity(args.len());
            for arg in args {
                if arg.trim().is_empty() {
                    return Err(Error::ScriptParse("push argument cannot be empty".into()));
                }
                parsed.push(parse_expr(arg.trim())?);
            }
            Expr::ArrayPush {
                target,
                args: parsed,
            }
        }
        "pop" => {
            if !args.is_empty() {
                return Err(Error::ScriptParse("pop does not take arguments".into()));
            }
            Expr::ArrayPop(target)
        }
        "shift" => {
            if !args.is_empty() {
                return Err(Error::ScriptParse("shift does not take arguments".into()));
            }
            Expr::ArrayShift(target)
        }
        "unshift" => {
            let mut parsed = Vec::with_capacity(args.len());
            for arg in args {
                if arg.trim().is_empty() {
                    return Err(Error::ScriptParse("unshift argument cannot be empty".into()));
                }
                parsed.push(parse_expr(arg.trim())?);
            }
            Expr::ArrayUnshift {
                target,
                args: parsed,
            }
        }
        "map" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "map requires exactly one callback argument".into(),
                ));
            }
            let callback = parse_array_callback_arg(args[0], 3, "array callback parameters")?;
            Expr::ArrayMap { target, callback }
        }
        "filter" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "filter requires exactly one callback argument".into(),
                ));
            }
            let callback = parse_array_callback_arg(args[0], 3, "array callback parameters")?;
            Expr::ArrayFilter { target, callback }
        }
        "reduce" => {
            if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "reduce requires callback and optional initial value".into(),
                ));
            }
            let callback = parse_array_callback_arg(args[0], 4, "array callback parameters")?;
            let initial = if args.len() == 2 {
                if args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "reduce initial value cannot be empty".into(),
                    ));
                }
                Some(Box::new(parse_expr(args[1].trim())?))
            } else {
                None
            };
            Expr::ArrayReduce {
                target,
                callback,
                initial,
            }
        }
        "forEach" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "forEach requires exactly one callback argument".into(),
                ));
            }
            let callback = parse_array_callback_arg(args[0], 3, "array callback parameters")?;
            Expr::ArrayForEach { target, callback }
        }
        "find" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "find requires exactly one callback argument".into(),
                ));
            }
            let callback = parse_array_callback_arg(args[0], 3, "array callback parameters")?;
            Expr::ArrayFind { target, callback }
        }
        "some" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "some requires exactly one callback argument".into(),
                ));
            }
            let callback = parse_array_callback_arg(args[0], 3, "array callback parameters")?;
            Expr::ArraySome { target, callback }
        }
        "every" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "every requires exactly one callback argument".into(),
                ));
            }
            let callback = parse_array_callback_arg(args[0], 3, "array callback parameters")?;
            Expr::ArrayEvery { target, callback }
        }
        "includes" => {
            if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "includes requires one or two arguments".into(),
                ));
            }
            if args.len() == 2 && args[1].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "includes fromIndex cannot be empty".into(),
                ));
            }
            Expr::ArrayIncludes {
                target,
                search: Box::new(parse_expr(args[0].trim())?),
                from_index: if args.len() == 2 {
                    Some(Box::new(parse_expr(args[1].trim())?))
                } else {
                    None
                },
            }
        }
        "slice" => {
            if args.len() > 2 {
                return Err(Error::ScriptParse(
                    "slice supports up to two arguments".into(),
                ));
            }
            let start = if !args.is_empty() {
                if args[0].trim().is_empty() {
                    return Err(Error::ScriptParse("slice start cannot be empty".into()));
                }
                Some(Box::new(parse_expr(args[0].trim())?))
            } else {
                None
            };
            let end = if args.len() == 2 {
                if args[1].trim().is_empty() {
                    return Err(Error::ScriptParse("slice end cannot be empty".into()));
                }
                Some(Box::new(parse_expr(args[1].trim())?))
            } else {
                None
            };
            Expr::ArraySlice { target, start, end }
        }
        "splice" => {
            if args.is_empty() || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "splice requires at least start index".into(),
                ));
            }
            let start = Box::new(parse_expr(args[0].trim())?);
            let delete_count = if args.len() >= 2 {
                if args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "splice deleteCount cannot be empty".into(),
                    ));
                }
                Some(Box::new(parse_expr(args[1].trim())?))
            } else {
                None
            };
            let mut items = Vec::new();
            for arg in args.iter().skip(2) {
                if arg.trim().is_empty() {
                    return Err(Error::ScriptParse("splice item cannot be empty".into()));
                }
                items.push(parse_expr(arg.trim())?);
            }
            Expr::ArraySplice {
                target,
                start,
                delete_count,
                items,
            }
        }
        "join" => {
            if args.len() > 1 {
                return Err(Error::ScriptParse(
                    "join supports at most one argument".into(),
                ));
            }
            let separator = if args.len() == 1 {
                if args[0].trim().is_empty() {
                    return Err(Error::ScriptParse("join separator cannot be empty".into()));
                }
                Some(Box::new(parse_expr(args[0].trim())?))
            } else {
                None
            };
            Expr::ArrayJoin { target, separator }
        }
        _ => return Ok(None),
    };

    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(expr))
}

fn parse_array_callback_arg(arg: &str, max_params: usize, label: &str) -> Result<ScriptHandler> {
    let callback_arg = strip_js_comments(arg);
    let mut callback_cursor = Cursor::new(callback_arg.as_str().trim());
    let (params, body) = parse_callback(&mut callback_cursor, max_params, label)?;
    callback_cursor.skip_ws();
    if !callback_cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported array callback: {}",
            arg.trim()
        )));
    }

    let stmts = if let Ok(expr) = parse_expr(body.trim()) {
        vec![Stmt::Return { value: Some(expr) }]
    } else {
        parse_block_statements(&body)?
    };

    Ok(ScriptHandler { params, stmts })
}

fn parse_string_method_expr(src: &str) -> Result<Option<Expr>> {
    let src = src.trim();
    let dots = collect_top_level_char_positions(src, b'.');
    for dot in dots.into_iter().rev() {
        let Some(base_src) = src.get(..dot) else {
            continue;
        };
        let base_src = base_src.trim();
        if base_src.is_empty() {
            continue;
        }
        let Some(tail_src) = src.get(dot + 1..) else {
            continue;
        };
        let tail_src = tail_src.trim();

        let mut cursor = Cursor::new(tail_src);
        let Some(method) = cursor.parse_identifier() else {
            continue;
        };
        cursor.skip_ws();
        if cursor.peek() != Some(b'(') {
            continue;
        }
        let args_src = cursor.read_balanced_block(b'(', b')')?;
        cursor.skip_ws();
        if !cursor.eof() {
            continue;
        }

        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };

        if !matches!(
            method.as_str(),
            "trim"
                | "trimStart"
                | "trimEnd"
                | "toUpperCase"
                | "toLowerCase"
                | "includes"
                | "startsWith"
                | "endsWith"
                | "slice"
                | "substring"
                | "split"
                | "replace"
                | "indexOf"
        ) {
            continue;
        }

        let base = Box::new(parse_expr(base_src)?);
        let expr = match method.as_str() {
            "trim" => {
                if !args.is_empty() {
                    return Err(Error::ScriptParse("trim does not take arguments".into()));
                }
                Expr::StringTrim {
                    value: base,
                    mode: StringTrimMode::Both,
                }
            }
            "trimStart" => {
                if !args.is_empty() {
                    return Err(Error::ScriptParse("trimStart does not take arguments".into()));
                }
                Expr::StringTrim {
                    value: base,
                    mode: StringTrimMode::Start,
                }
            }
            "trimEnd" => {
                if !args.is_empty() {
                    return Err(Error::ScriptParse("trimEnd does not take arguments".into()));
                }
                Expr::StringTrim {
                    value: base,
                    mode: StringTrimMode::End,
                }
            }
            "toUpperCase" => {
                if !args.is_empty() {
                    return Err(Error::ScriptParse(
                        "toUpperCase does not take arguments".into(),
                    ));
                }
                Expr::StringToUpperCase(base)
            }
            "toLowerCase" => {
                if !args.is_empty() {
                    return Err(Error::ScriptParse(
                        "toLowerCase does not take arguments".into(),
                    ));
                }
                Expr::StringToLowerCase(base)
            }
            "includes" => {
                if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "String.includes requires one or two arguments".into(),
                    ));
                }
                if args.len() == 2 && args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "String.includes position cannot be empty".into(),
                    ));
                }
                Expr::StringIncludes {
                    value: base,
                    search: Box::new(parse_expr(args[0].trim())?),
                    position: if args.len() == 2 {
                        Some(Box::new(parse_expr(args[1].trim())?))
                    } else {
                        None
                    },
                }
            }
            "startsWith" => {
                if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "startsWith requires one or two arguments".into(),
                    ));
                }
                if args.len() == 2 && args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "startsWith position cannot be empty".into(),
                    ));
                }
                Expr::StringStartsWith {
                    value: base,
                    search: Box::new(parse_expr(args[0].trim())?),
                    position: if args.len() == 2 {
                        Some(Box::new(parse_expr(args[1].trim())?))
                    } else {
                        None
                    },
                }
            }
            "endsWith" => {
                if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "endsWith requires one or two arguments".into(),
                    ));
                }
                if args.len() == 2 && args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "endsWith length argument cannot be empty".into(),
                    ));
                }
                Expr::StringEndsWith {
                    value: base,
                    search: Box::new(parse_expr(args[0].trim())?),
                    length: if args.len() == 2 {
                        Some(Box::new(parse_expr(args[1].trim())?))
                    } else {
                        None
                    },
                }
            }
            "slice" => {
                if args.len() > 2 {
                    return Err(Error::ScriptParse(
                        "String.slice supports up to two arguments".into(),
                    ));
                }
                let start = if !args.is_empty() {
                    if args[0].trim().is_empty() {
                        return Err(Error::ScriptParse(
                            "String.slice start cannot be empty".into(),
                        ));
                    }
                    Some(Box::new(parse_expr(args[0].trim())?))
                } else {
                    None
                };
                let end = if args.len() == 2 {
                    if args[1].trim().is_empty() {
                        return Err(Error::ScriptParse(
                            "String.slice end cannot be empty".into(),
                        ));
                    }
                    Some(Box::new(parse_expr(args[1].trim())?))
                } else {
                    None
                };
                Expr::StringSlice {
                    value: base,
                    start,
                    end,
                }
            }
            "substring" => {
                if args.len() > 2 {
                    return Err(Error::ScriptParse(
                        "substring supports up to two arguments".into(),
                    ));
                }
                let start = if !args.is_empty() {
                    if args[0].trim().is_empty() {
                        return Err(Error::ScriptParse(
                            "substring start cannot be empty".into(),
                        ));
                    }
                    Some(Box::new(parse_expr(args[0].trim())?))
                } else {
                    None
                };
                let end = if args.len() == 2 {
                    if args[1].trim().is_empty() {
                        return Err(Error::ScriptParse(
                            "substring end cannot be empty".into(),
                        ));
                    }
                    Some(Box::new(parse_expr(args[1].trim())?))
                } else {
                    None
                };
                Expr::StringSubstring {
                    value: base,
                    start,
                    end,
                }
            }
            "split" => {
                if args.len() > 2 {
                    return Err(Error::ScriptParse(
                        "split supports up to two arguments".into(),
                    ));
                }
                let separator = if !args.is_empty() {
                    if args[0].trim().is_empty() {
                        return Err(Error::ScriptParse(
                            "split separator cannot be empty expression".into(),
                        ));
                    }
                    Some(Box::new(parse_expr(args[0].trim())?))
                } else {
                    None
                };
                let limit = if args.len() == 2 {
                    if args[1].trim().is_empty() {
                        return Err(Error::ScriptParse(
                            "split limit cannot be empty".into(),
                        ));
                    }
                    Some(Box::new(parse_expr(args[1].trim())?))
                } else {
                    None
                };
                Expr::StringSplit {
                    value: base,
                    separator,
                    limit,
                }
            }
            "replace" => {
                if args.len() != 2 || args[0].trim().is_empty() || args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "replace requires exactly two arguments".into(),
                    ));
                }
                Expr::StringReplace {
                    value: base,
                    from: Box::new(parse_expr(args[0].trim())?),
                    to: Box::new(parse_expr(args[1].trim())?),
                }
            }
            "indexOf" => {
                if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "indexOf requires one or two arguments".into(),
                    ));
                }
                if args.len() == 2 && args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "indexOf position cannot be empty".into(),
                    ));
                }
                Expr::StringIndexOf {
                    value: base,
                    search: Box::new(parse_expr(args[0].trim())?),
                    position: if args.len() == 2 {
                        Some(Box::new(parse_expr(args[1].trim())?))
                    } else {
                        None
                    },
                }
            }
            _ => unreachable!(),
        };

        return Ok(Some(expr));
    }

    Ok(None)
}

fn parse_date_method_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    let Some(target) = cursor.parse_identifier() else {
        return Ok(None);
    };
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    let Some(method) = cursor.parse_identifier() else {
        return Ok(None);
    };
    cursor.skip_ws();
    if cursor.peek() != Some(b'(') {
        return Ok(None);
    }

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let raw_args = split_top_level_by_char(&args_src, b',');
    let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
        Vec::new()
    } else {
        raw_args
    };

    let expr = match method.as_str() {
        "getTime" => {
            if !args.is_empty() {
                return Err(Error::ScriptParse("getTime does not take arguments".into()));
            }
            Expr::DateGetTime(target)
        }
        "setTime" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "setTime requires exactly one argument".into(),
                ));
            }
            Expr::DateSetTime {
                target,
                value: Box::new(parse_expr(args[0].trim())?),
            }
        }
        "toISOString" => {
            if !args.is_empty() {
                return Err(Error::ScriptParse(
                    "toISOString does not take arguments".into(),
                ));
            }
            Expr::DateToIsoString(target)
        }
        "getFullYear" => {
            if !args.is_empty() {
                return Err(Error::ScriptParse(
                    "getFullYear does not take arguments".into(),
                ));
            }
            Expr::DateGetFullYear(target)
        }
        "getMonth" => {
            if !args.is_empty() {
                return Err(Error::ScriptParse("getMonth does not take arguments".into()));
            }
            Expr::DateGetMonth(target)
        }
        "getDate" => {
            if !args.is_empty() {
                return Err(Error::ScriptParse("getDate does not take arguments".into()));
            }
            Expr::DateGetDate(target)
        }
        "getHours" => {
            if !args.is_empty() {
                return Err(Error::ScriptParse("getHours does not take arguments".into()));
            }
            Expr::DateGetHours(target)
        }
        "getMinutes" => {
            if !args.is_empty() {
                return Err(Error::ScriptParse(
                    "getMinutes does not take arguments".into(),
                ));
            }
            Expr::DateGetMinutes(target)
        }
        "getSeconds" => {
            if !args.is_empty() {
                return Err(Error::ScriptParse(
                    "getSeconds does not take arguments".into(),
                ));
            }
            Expr::DateGetSeconds(target)
        }
        _ => return Ok(None),
    };

    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(expr))
}

fn collect_top_level_char_positions(src: &str, target: u8) -> Vec<usize> {
    let bytes = src.as_bytes();
    let mut out = Vec::new();
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
                        out.push(i);
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

    out
}

fn parse_set_timeout_expr(src: &str) -> Result<Option<(TimerInvocation, Expr)>> {
    let mut cursor = Cursor::new(src);
    let Some((handler, delay_ms)) = parse_set_timeout_call(&mut cursor)? else {
        return Ok(None);
    };
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some((handler, delay_ms)))
}

fn parse_set_interval_expr(src: &str) -> Result<Option<(TimerInvocation, Expr)>> {
    let mut cursor = Cursor::new(src);
    let Some((handler, delay_ms)) = parse_set_interval_call(&mut cursor)? else {
        return Ok(None);
    };
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some((handler, delay_ms)))
}

fn parse_queue_microtask_expr(src: &str) -> Result<Option<ScriptHandler>> {
    let mut cursor = Cursor::new(src);
    let handler = parse_queue_microtask_call(&mut cursor)?;
    cursor.skip_ws();
    if cursor.eof() {
        Ok(handler)
    } else {
        Ok(None)
    }
}

fn parse_queue_microtask_stmt(stmt: &str) -> Result<Option<Stmt>> {
    let stmt = stmt.trim();
    let mut cursor = Cursor::new(stmt);
    let Some(handler) = parse_queue_microtask_call(&mut cursor)? else {
        return Ok(None);
    };

    cursor.skip_ws();
    cursor.consume_byte(b';');
    cursor.skip_ws();
    if !cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported queueMicrotask statement tail: {stmt}"
        )));
    }

    Ok(Some(Stmt::QueueMicrotask { handler }))
}

fn parse_queue_microtask_call(cursor: &mut Cursor<'_>) -> Result<Option<ScriptHandler>> {
    cursor.skip_ws();
    if !cursor.consume_ascii("queueMicrotask") {
        return Ok(None);
    }

    cursor.skip_ws();
    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.is_empty() {
        return Err(Error::ScriptParse(
            "queueMicrotask requires 1 argument".into(),
        ));
    }
    if args.len() != 1 {
        return Err(Error::ScriptParse(
            "queueMicrotask supports only 1 argument".into(),
        ));
    }

    let callback_arg = strip_js_comments(args[0]);
    let mut callback_cursor = Cursor::new(callback_arg.as_str().trim());
    let (params, body) = parse_callback(&mut callback_cursor, 1, "callback parameters")?;
    callback_cursor.skip_ws();
    if !callback_cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported queueMicrotask callback: {}",
            args[0].trim()
        )));
    }

    Ok(Some(ScriptHandler {
        params,
        stmts: parse_block_statements(&body)?,
    }))
}

fn parse_promise_then_expr(src: &str) -> Result<Option<ScriptHandler>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    if !cursor.consume_ascii("Promise") {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("resolve") {
        return Ok(None);
    }
    cursor.skip_ws();
    let _ = cursor.read_balanced_block(b'(', b')')?;
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("then") {
        return Ok(None);
    }
    cursor.skip_ws();

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    let args = split_top_level_by_char(&args_src, b',');
    if args.is_empty() {
        return Err(Error::ScriptParse("Promise.then requires 1 argument".into()));
    }
    if args.len() != 1 {
        return Err(Error::ScriptParse(
            "Promise.resolve().then supports only 1 callback argument".into(),
        ));
    }

    let callback_arg = strip_js_comments(args[0]);
    let mut callback_cursor = Cursor::new(callback_arg.as_str().trim());
    let (params, body) = parse_callback(&mut callback_cursor, 1, "callback parameters")?;
    callback_cursor.skip_ws();
    if !callback_cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported Promise.then callback: {}",
            args[0].trim()
        )));
    }

    Ok(Some(ScriptHandler {
        params,
        stmts: parse_block_statements(&body)?,
    }))
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
        ("readonly", None) | ("readOnly", None) => DomProp::Readonly,
        ("required", None) => DomProp::Required,
        ("disabled", None) => DomProp::Disabled,
        ("textContent", None) => DomProp::TextContent,
        ("innerHTML", None) => DomProp::InnerHtml,
        ("className", None) => DomProp::ClassName,
        ("id", None) => DomProp::Id,
        ("name", None) => DomProp::Name,
        ("offsetWidth", None) => DomProp::OffsetWidth,
        ("offsetHeight", None) => DomProp::OffsetHeight,
        ("offsetLeft", None) => DomProp::OffsetLeft,
        ("offsetTop", None) => DomProp::OffsetTop,
        ("scrollWidth", None) => DomProp::ScrollWidth,
        ("scrollHeight", None) => DomProp::ScrollHeight,
        ("scrollLeft", None) => DomProp::ScrollLeft,
        ("scrollTop", None) => DomProp::ScrollTop,
        ("activeElement", None) if matches!(target, DomQuery::DocumentRoot) => {
            DomProp::ActiveElement
        }
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

fn parse_dom_matches_expr(src: &str) -> Result<Option<(DomQuery, String)>> {
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
    if !cursor.consume_ascii("matches") {
        return Ok(None);
    }
    cursor.skip_ws();
    cursor.expect_byte(b'(')?;
    cursor.skip_ws();
    let selector = cursor.parse_string_literal()?;
    cursor.skip_ws();
    cursor.expect_byte(b')')?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    Ok(Some((target, selector)))
}

fn parse_dom_closest_expr(src: &str) -> Result<Option<(DomQuery, String)>> {
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
    if !cursor.consume_ascii("closest") {
        return Ok(None);
    }
    cursor.skip_ws();
    cursor.expect_byte(b'(')?;
    cursor.skip_ws();
    let selector = cursor.parse_string_literal()?;
    cursor.skip_ws();
    cursor.expect_byte(b')')?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    Ok(Some((target, selector)))
}

fn parse_dom_computed_style_property_expr(src: &str) -> Result<Option<(DomQuery, String)>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    if !cursor.consume_ascii("getComputedStyle") {
        return Ok(None);
    }
    cursor.skip_ws();
    cursor.expect_byte(b'(')?;
    cursor.skip_ws();
    let target = parse_element_target(&mut cursor)?;
    cursor.skip_ws();
    cursor.expect_byte(b')')?;
    cursor.skip_ws();
    cursor.expect_byte(b'.')?;
    cursor.skip_ws();
    if !cursor.consume_ascii("getPropertyValue") {
        return Ok(None);
    }
    cursor.skip_ws();
    cursor.expect_byte(b'(')?;
    cursor.skip_ws();
    let property = cursor.parse_string_literal()?;
    cursor.skip_ws();
    cursor.expect_byte(b')')?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some((target, property)))
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
        ("target", Some("name")) => EventExprProp::TargetName,
        ("currentTarget", Some("name")) => EventExprProp::CurrentTargetName,
        ("defaultPrevented", None) => EventExprProp::DefaultPrevented,
        ("isTrusted", None) => EventExprProp::IsTrusted,
        ("bubbles", None) => EventExprProp::Bubbles,
        ("cancelable", None) => EventExprProp::Cancelable,
        ("target", Some("id")) => EventExprProp::TargetId,
        ("currentTarget", Some("id")) => EventExprProp::CurrentTargetId,
        ("eventPhase", None) => EventExprProp::EventPhase,
        ("timeStamp", None) => EventExprProp::TimeStamp,
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

fn parse_query_selector_all_length_expr(src: &str) -> Result<Option<DomQuery>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    let target = match parse_element_target(&mut cursor) {
        Ok(target) => target,
        Err(_) => return Ok(None),
    };
    let is_list_target = matches!(
        target,
        DomQuery::BySelectorAll { .. } | DomQuery::QuerySelectorAll { .. } | DomQuery::Var(_)
    );
    if !is_list_target {
        return Ok(None);
    }

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

    Ok(Some(target))
}

fn parse_form_elements_length_expr(src: &str) -> Result<Option<DomQuery>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    let form = match parse_form_elements_base(&mut cursor) {
        Ok(target) => target,
        Err(_) => return Ok(None),
    };
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("elements") {
        return Ok(None);
    }
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
    Ok(Some(form))
}

fn parse_new_form_data_expr(src: &str) -> Result<Option<DomQuery>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    let Some(form) = parse_new_form_data_target(&mut cursor)? else {
        return Ok(None);
    };
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(form))
}

fn parse_form_data_get_expr(src: &str) -> Result<Option<(FormDataSource, String)>> {
    parse_form_data_method_expr(src, "get")
}

fn parse_form_data_has_expr(src: &str) -> Result<Option<(FormDataSource, String)>> {
    parse_form_data_method_expr(src, "has")
}

fn parse_form_data_get_all_length_expr(src: &str) -> Result<Option<(FormDataSource, String)>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    let Some(source) = parse_form_data_source(&mut cursor)? else {
        return Ok(None);
    };

    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    let Some(method) = cursor.parse_identifier() else {
        return Ok(None);
    };
    if method != "getAll" {
        return Ok(None);
    }

    cursor.skip_ws();
    cursor.expect_byte(b'(')?;
    cursor.skip_ws();
    let name = cursor.parse_string_literal()?;
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

    Ok(Some((source, name)))
}

fn parse_form_data_method_expr(
    src: &str,
    method: &str,
) -> Result<Option<(FormDataSource, String)>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    let Some(source) = parse_form_data_source(&mut cursor)? else {
        return Ok(None);
    };

    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    let Some(actual_method) = cursor.parse_identifier() else {
        return Ok(None);
    };
    if actual_method != method {
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

    Ok(Some((source, name)))
}

fn parse_form_data_source(cursor: &mut Cursor<'_>) -> Result<Option<FormDataSource>> {
    if let Some(form) = parse_new_form_data_target(cursor)? {
        return Ok(Some(FormDataSource::NewForm(form)));
    }

    if let Some(var_name) = cursor.parse_identifier() {
        return Ok(Some(FormDataSource::Var(var_name)));
    }

    Ok(None)
}

fn parse_new_form_data_target(cursor: &mut Cursor<'_>) -> Result<Option<DomQuery>> {
    cursor.skip_ws();
    let start = cursor.pos();

    if !cursor.consume_ascii("new") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            cursor.set_pos(start);
            return Ok(None);
        }
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("FormData") {
        cursor.set_pos(start);
        return Ok(None);
    }
    cursor.skip_ws();

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(args_src.trim(), b',');
    if args.len() != 1 {
        return Err(Error::ScriptParse(
            "new FormData requires exactly one argument".into(),
        ));
    }

    let arg = args[0].trim();
    let mut arg_cursor = Cursor::new(arg);
    arg_cursor.skip_ws();
    let form = parse_form_elements_base(&mut arg_cursor)?;
    arg_cursor.skip_ws();
    if !arg_cursor.eof() {
        return Err(Error::ScriptParse(format!(
            "unsupported FormData argument: {arg}"
        )));
    }

    Ok(Some(form))
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

fn decode_html_character_references(src: &str) -> String {
    if !src.contains('&') {
        return src.to_string();
    }

    fn is_entity_token_char(ch: char) -> bool {
        ch.is_ascii_alphanumeric() || ch == '#' || ch == 'x' || ch == 'X'
    }

    fn decode_numeric(value: &str) -> Option<char> {
        let codepoint = if let Some(hex) = value.strip_prefix("x").or_else(|| value.strip_prefix("X")) {
            u32::from_str_radix(hex, 16).ok()?
        } else {
            u32::from_str_radix(value, 10).ok()?
        };
        char::from_u32(codepoint)
    }

    fn decode_named(value: &str) -> Option<char> {
        match value {
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" => Some('\''),
            "nbsp" => Some('\u{00A0}'),
            "divide" => Some('÷'),
            "times" => Some('×'),
            "ensp" => Some('\u{2002}'),
            "emsp" => Some('\u{2003}'),
            "thinsp" => Some('\u{2009}'),
            "copy" => Some('©'),
            "reg" => Some('®'),
            "trade" => Some('™'),
            "euro" => Some('€'),
            "pound" => Some('£'),
            "yen" => Some('¥'),
            "laquo" => Some('«'),
            "raquo" => Some('»'),
            "ldquo" => Some('“'),
            "rdquo" => Some('”'),
            "lsquo" => Some('‘'),
            "rsquo" => Some('’'),
            "hellip" => Some('…'),
            "middot" => Some('·'),
            "frac14" => Some('¼'),
            "frac12" => Some('½'),
            "frac34" => Some('¾'),
            "frac13" => Some('\u{2153}'),
            "frac15" => Some('\u{2155}'),
            "frac16" => Some('\u{2159}'),
            "frac18" => Some('\u{215B}'),
            "frac23" => Some('\u{2154}'),
            "frac25" => Some('\u{2156}'),
            "frac35" => Some('\u{2157}'),
            "frac38" => Some('\u{215C}'),
            "frac45" => Some('\u{2158}'),
            "frac56" => Some('\u{215A}'),
            "frac58" => Some('\u{215E}'),
            "not" => Some('¬'),
            "deg" => Some('°'),
            "plusmn" => Some('±'),
            "larr" => Some('←'),
            "rarr" => Some('→'),
            _ => None,
        }
    }

    let mut out = String::with_capacity(src.len());
    let mut i = 0usize;

    while i < src.len() {
        let ch = src[i..].chars().next().unwrap_or_default();
        if ch != '&' {
            out.push(ch);
            i += ch.len_utf8();
            continue;
        }

        let tail = &src[i + 1..];
        let mut semicolon_end = None;
        if let Some(semicolon_pos) = tail.find(';') {
            match tail.find('&') {
                Some(next_amp_pos) if next_amp_pos < semicolon_pos => {}
                _ => semicolon_end = Some(semicolon_pos),
            }
        }

        let Some(end_offset) = semicolon_end else {
            let entity_end = tail
                .char_indices()
                .find_map(|(idx, ch)| if is_entity_token_char(ch) { None } else { Some(idx) })
                .unwrap_or(tail.len());

            if entity_end == 0 {
                out.push('&');
                i += 1;
                continue;
            }

            let raw = &tail[..entity_end];
            let decoded = if let Some(rest) = raw.strip_prefix('#') {
                decode_numeric(rest)
            } else {
                decode_named(raw)
            };

            if let Some(value) = decoded {
                out.push(value);
                i += entity_end + 1;
            } else {
                out.push('&');
                i += 1;
            }
            continue;
        };

        let raw = &tail[..end_offset];
        let decoded = if let Some(rest) = raw.strip_prefix('#') {
            decode_numeric(rest)
        } else {
            decode_named(raw)
        };

        if let Some(value) = decoded {
            out.push(value);
            i += end_offset + 2;
        } else {
            out.push('&');
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

fn find_top_level_assignment(src: &str) -> Option<(usize, usize)> {
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
                b'=' => {
                    if paren == 0 && bracket == 0 && brace == 0 {
                        if i + 1 < bytes.len() && bytes[i + 1] == b'=' {
                            if i + 2 < bytes.len() && bytes[i + 2] == b'=' {
                                i += 2;
                            } else {
                                i += 1;
                            }
                            } else if i >= 2 && &bytes[i - 2..=i] == b"**=" {
                                return Some((i - 2, 3));
                            } else if i >= 3 && &bytes[i - 3..=i] == b">>>=" {
                                return Some((i - 3, 4));
                            } else if i >= 2 && &bytes[i - 2..=i] == b"<<=" {
                                return Some((i - 2, 3));
                            } else if i >= 2 && &bytes[i - 2..=i] == b">>=" {
                                return Some((i - 2, 3));
                        } else if i > 0
                            && matches!(
                                bytes[i - 1],
                                b'+' | b'-' | b'*' | b'/' | b'%' | b'&' | b'|' | b'^'
                            )
                        {
                            return Some((i - 1, 2));
                        } else {
                            return Some((i, 1));
                        }
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
                                if op_bytes.iter().all(|b| b.is_ascii_alphabetic()) {
                                    if i > 0 && is_ident_char(bytes[i - 1]) {
                                        continue;
                                    }
                                    if i + op_bytes.len() < bytes.len()
                                        && is_ident_char(bytes[i + op_bytes.len()])
                                    {
                                        continue;
                                    }
                                } else if op.len() == 1 && (op == &"<" || op == &">") {
                                    let prev = if i == 0 {
                                        None
                                    } else {
                                        Some(bytes[i - 1])
                                    };
                                    let next = bytes.get(i + 1).copied();
                                    if prev == Some(b'<')
                                        || prev == Some(b'>')
                                        || next == Some(b'<')
                                        || next == Some(b'>')
                                    {
                                        continue;
                                    }
                                }
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
                dom.create_text(parent, decode_html_character_references(text));
            }
        }
    }

    dom.initialize_form_control_values()?;
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
        return Ok(decode_html_character_references(&value));
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
    Ok(decode_html_character_references(&value))
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
    fn is_ident_separator(byte: u8) -> bool {
        !byte.is_ascii_alphanumeric()
    }

    let mut i = from;
    enum State {
        Normal,
        Single,
        Double,
        Template,
    }
    let mut state = State::Normal;

    while i < bytes.len() {
        let b = bytes[i];

        match state {
            State::Normal => {
                if b == b'\'' {
                    state = State::Single;
                    i += 1;
                    continue;
                }
                if b == b'"' {
                    state = State::Double;
                    i += 1;
                    continue;
                }
                if b == b'`' {
                    state = State::Template;
                    i += 1;
                    continue;
                }
                if i + 1 < bytes.len() && b == b'/' && bytes[i + 1] == b'/' {
                    i += 2;
                    while i < bytes.len() && bytes[i] != b'\n' {
                        i += 1;
                    }
                    continue;
                }
                if i + 1 < bytes.len() && b == b'/' && bytes[i + 1] == b'*' {
                    i += 2;
                    while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                        i += 1;
                    }
                    if i + 1 < bytes.len() {
                        i += 2;
                    } else {
                        i = bytes.len();
                    }
                    continue;
                }
                if b == b'<' && bytes.get(i + 1) == Some(&b'/') {
                    let mut j = i + 2;
                    while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                        j += 1;
                    }
                    let tag_end = j + tag.len();
                    if tag_end <= bytes.len() {
                        let mut matched = true;
                        for k in 0..tag.len() {
                            if bytes[j + k].to_ascii_lowercase() != tag[k].to_ascii_lowercase() {
                                matched = false;
                                break;
                            }
                        }
                        if matched {
                            let after = j + tag.len();
                            if after >= bytes.len() || is_ident_separator(bytes[after]) {
                                return Some(i);
                            }
                        }
                    }
                }
                i += 1;
            }
            State::Single => {
                if b == b'\\' {
                    i += 2;
                } else {
                    if b == b'\'' {
                        state = State::Normal;
                    }
                    i += 1;
                }
            }
            State::Double => {
                if b == b'\\' {
                    i += 2;
                } else {
                    if b == b'"' {
                        state = State::Normal;
                    }
                    i += 1;
                }
            }
            State::Template => {
                if b == b'\\' {
                    i += 2;
                } else {
                    if b == b'`' {
                        state = State::Normal;
                    }
                    i += 1;
                }
            }
        }
    }
    None
}

fn is_form_control(dom: &Dom, node_id: NodeId) -> bool {
    let Some(element) = dom.element(node_id) else {
        return false;
    };

    element.tag_name.eq_ignore_ascii_case("input")
        || element.tag_name.eq_ignore_ascii_case("select")
        || element.tag_name.eq_ignore_ascii_case("textarea")
        || element.tag_name.eq_ignore_ascii_case("button")
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
        self.skip_ws_and_comments()
    }

    fn skip_ws_and_comments(&mut self) {
        loop {
            self.skip_plain_ws();
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

    fn skip_plain_ws(&mut self) {
        while let Some(b) = self.peek() {
            if b.is_ascii_whitespace() {
                self.i += 1;
            } else {
                break;
            }
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
    fn mock_window_supports_multiple_pages() -> Result<()> {
        let mut win = MockWindow::new();
        win.open_page(
            "https://app.local/a",
            r#"
            <button id='btn'>A</button>
            <p id='result'></p>
            <script>
              document.getElementById('btn').addEventListener('click', () => {
                document.getElementById('result').textContent = 'A';
              });
            </script>
        "#,
        )?;

        win.open_page(
            "https://app.local/b",
            r#"
            <button id='btn'>B</button>
            <p id='result'></p>
            <script>
              document.getElementById('btn').addEventListener('click', () => {
                document.getElementById('result').textContent = 'B';
              });
            </script>
        "#,
        )?;

        win.switch_to("https://app.local/a")?;
        win.click("#btn")?;
        win.assert_text("#result", "A")?;

        win.switch_to("https://app.local/b")?;
        win.assert_text("#result", "")?;
        win.click("#btn")?;
        win.assert_text("#result", "B")?;

        win.switch_to("https://app.local/a")?;
        win.assert_text("#result", "A")?;
        Ok(())
    }

    #[test]
    fn window_aliases_document_in_script_parser() -> Result<()> {
        let html = r#"
        <p id='result'>before</p>
        <script>
          window.document.getElementById('result').textContent = 'after';
        </script>
        "#;

        let h = Harness::from_html(html)?;
        h.assert_text("#result", "after")?;
        Ok(())
    }

    #[test]
    fn html_entities_in_text_nodes_are_decoded() -> Result<()> {
        let html = "<p id='result'>&lt;A &amp; B&gt;&nbsp;&copy;</p>";
        let h = Harness::from_html(html)?;
        h.assert_text("#result", "<A & B>\u{00A0}©")?;
        Ok(())
    }

    #[test]
    fn html_entities_in_attribute_values_are_decoded() -> Result<()> {
        let html = r#"
        <div id='result' data-value='a&amp;b&nbsp;&#x3c;'></div>
        <script>
          document.getElementById('result').textContent =
            document.getElementById('result').getAttribute('data-value');
        </script>
        "#;

        let h = Harness::from_html(html)?;
        h.assert_text("#result", "a&b\u{00A0}<")?;
        Ok(())
    }

    #[test]
    fn html_entities_in_inner_html_are_decoded() -> Result<()> {
        let html = r#"
        <div id='host'></div>
        <p id='result'></p>
        <script>
          document.getElementById('host').innerHTML =
            '<span id="value">a&amp;b&nbsp;</span>';
          document.getElementById('result').textContent =
            document.getElementById('value').textContent;
        </script>
        "#;

        let h = Harness::from_html(html)?;
        h.assert_text("#result", "a&b\u{00A0}")?;
        Ok(())
    }

    #[test]
    fn html_entities_without_trailing_semicolon_are_decoded() -> Result<()> {
        let html =
            "<p id='result'>&lt;A &amp B &gt C&copy D&thinsp;E&ensp;F&emsp;G&frac12;H</p>";

        let h = Harness::from_html(html)?;
        h.assert_text("#result", "<A & B > C© D\u{2009}E\u{2002}F\u{2003}G½H")?;
        Ok(())
    }

    #[test]
    fn html_entities_known_named_references_are_decoded() -> Result<()> {
        let html = "<p id='result'>&larr;&rarr;</p>";

        let h = Harness::from_html(html)?;
        h.assert_text("#result", "←→")?;
        Ok(())
    }

    #[test]
    fn html_entities_more_named_references_are_decoded() -> Result<()> {
        let html = "<p id='result'>&pound;&times;&divide;&laquo;&raquo;&frac13;&frac15;&frac16;&frac18;&frac23;&frac25;&frac34;&frac35;&frac38;&frac45;&frac56;&frac58;</p>";

        let h = Harness::from_html(html)?;
        h.assert_text(
            "#result",
            "\u{00A3}\u{00D7}\u{00F7}\u{00AB}\u{00BB}\u{2153}\u{2155}\u{2159}\u{215B}\u{2154}\u{2156}\u{00BE}\u{2157}\u{215C}\u{2158}\u{215A}\u{215E}",
        )?;
        Ok(())
    }

    #[test]
    fn html_entities_unknown_reference_boundary_cases_are_preserved() -> Result<()> {
        let html = "<p id='result'>&frac12x;&frac34;&poundfoo;&pound;&frac12abc;</p>";

        let h = Harness::from_html(html)?;
        h.assert_text("#result", "&frac12x;¾&poundfoo;£&frac12abc;")?;
        Ok(())
    }

    #[test]
    fn html_entities_unknown_named_references_are_not_decoded() -> Result<()> {
        let html = "<p id='result'>&nopenvelope;&copy;</p>";

        let h = Harness::from_html(html)?;
        h.assert_text("#result", "&nopenvelope;©")?;
        Ok(())
    }

    #[test]
    fn html_entities_without_semicolon_hex_and_decimal_numeric_are_decoded() -> Result<()> {
        let html = "<p id='result'>&#38&#60&#x3C&#x3e</p>";

        let h = Harness::from_html(html)?;
        h.assert_text("#result", "&<<>")?;
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
    fn form_elements_length_and_index_work() -> Result<()> {
        let html = r#"
        <form id='f'>
          <input id='name' value='N'>
          <textarea id='bio'>B</textarea>
          <button id='ok' type='button'>OK</button>
        </form>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const form = document.getElementById('f');
            document.getElementById('result').textContent =
              form.elements.length + ':' +
              form.elements[0].id + ':' +
              form.elements[1].id + ':' +
              form.elements[2].id;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "3:name:bio:ok")?;
        Ok(())
    }

    #[test]
    fn form_elements_index_supports_direct_property_access() -> Result<()> {
        let html = r#"
        <form id='f'>
          <input id='a' value='X'>
          <input id='b' value='Y'>
        </form>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent =
              document.getElementById('f').elements[1].value;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "Y")?;
        Ok(())
    }

    #[test]
    fn form_elements_index_supports_expression() -> Result<()> {
        let html = r#"
        <form id='f'>
          <input id='a' value='X'>
          <input id='b' value='Y'>
          <input id='c' value='Z'>
        </form>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const form = document.getElementById('f');
            const index = 1;
            const value = form.elements[index + 1].value;
            document.getElementById('result').textContent = value;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "Z")?;
        Ok(())
    }

    #[test]
    fn form_elements_out_of_range_returns_runtime_error() -> Result<()> {
        let html = r#"
        <form id='f'>
          <input id='a' value='X'>
        </form>
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('f').elements[5].id;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        let err = h.click("#btn").expect_err("out-of-range index should fail");
        match err {
            Error::ScriptRuntime(msg) => {
                assert!(msg.contains("elements[5]"));
                assert!(msg.contains("returned null"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
        Ok(())
    }

    #[test]
    fn textarea_initial_value_is_loaded_from_markup_text() -> Result<()> {
        let html = r#"
        <textarea id='bio' name='bio'>HELLO</textarea>
        "#;

        let h = Harness::from_html(html)?;
        h.assert_value("#bio", "HELLO")?;
        Ok(())
    }

    #[test]
    fn form_data_get_and_has_work_with_form_controls() -> Result<()> {
        let html = r#"
        <form id='f'>
          <input id='name' name='name' value='Taro'>
          <input id='agree' name='agree' type='checkbox' checked>
          <input id='skip' name='skip' type='checkbox'>
          <input id='disabled' name='disabled' value='x' disabled>
          <button id='submit' name='submit' type='submit' value='go'>Go</button>
        </form>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const form = document.getElementById('f');
            const fd = new FormData(form);
            document.getElementById('result').textContent =
              fd.get('name') + ':' +
              fd.get('agree') + ':' +
              fd.has('skip') + ':' +
              fd.has('disabled') + ':' +
              fd.has('submit');
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "Taro:on:false:false:false")?;
        Ok(())
    }

    #[test]
    fn form_data_uses_textarea_and_select_initial_values() -> Result<()> {
        let html = r#"
        <form id='f'>
          <textarea id='bio' name='bio'>HELLO</textarea>
          <select id='kind' name='kind'>
            <option id='k1' value='A'>Alpha</option>
            <option id='k2' selected>Beta</option>
          </select>
          <select id='city' name='city'>
            <option id='c1' value='tokyo'>Tokyo</option>
            <option id='c2' value='osaka'>Osaka</option>
          </select>
        </form>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const fd = new FormData(document.getElementById('f'));
            document.getElementById('result').textContent =
              fd.get('bio') + ':' + fd.get('kind') + ':' + fd.get('city');
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "HELLO:Beta:tokyo")?;
        Ok(())
    }

    #[test]
    fn form_data_reflects_option_selected_attribute_mutation() -> Result<()> {
        let html = r#"
        <form id='f'>
          <select id='kind' name='kind'>
            <option id='k1' selected value='A'>Alpha</option>
            <option id='k2' value='B'>Beta</option>
          </select>
        </form>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('k1').removeAttribute('selected');
            document.getElementById('k2').setAttribute('selected', 'true');
            const fd = new FormData(document.getElementById('f'));
            document.getElementById('result').textContent = fd.get('kind');
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "B")?;
        Ok(())
    }

    #[test]
    fn select_value_assignment_updates_selected_option_and_form_data() -> Result<()> {
        let html = r#"
        <form id='f'>
          <select id='kind' name='kind'>
            <option id='k1' selected value='A'>Alpha</option>
            <option id='k2' value='B'>Beta</option>
          </select>
        </form>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const sel = document.getElementById('kind');
            sel.value = 'B';
            const fd = new FormData(document.getElementById('f'));
            document.getElementById('result').textContent =
              fd.get('kind') + ':' +
              document.getElementById('k1').hasAttribute('selected') + ':' +
              document.getElementById('k2').hasAttribute('selected') + ':' +
              sel.value;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "B:false:true:B")?;
        Ok(())
    }

    #[test]
    fn select_value_assignment_can_match_option_text_without_value_attribute() -> Result<()> {
        let html = r#"
        <form id='f'>
          <select id='kind' name='kind'>
            <option id='k1'>Alpha</option>
            <option id='k2'>Beta</option>
          </select>
        </form>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const sel = document.getElementById('kind');
            sel.value = 'Beta';
            const fd = new FormData(document.getElementById('f'));
            document.getElementById('result').textContent =
              fd.get('kind') + ':' +
              sel.value + ':' +
              document.getElementById('k1').hasAttribute('selected') + ':' +
              document.getElementById('k2').hasAttribute('selected');
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "Beta:Beta:false:true")?;
        Ok(())
    }

    #[test]
    fn form_data_inline_constructor_call_works() -> Result<()> {
        let html = r#"
        <form id='f'>
          <input id='name' name='name' value='Hanako'>
        </form>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent =
              new FormData(document.getElementById('f')).get('name') + ':' +
              new FormData(document.getElementById('f')).has('missing') + ':' +
              new FormData(document.getElementById('f')).get('missing');
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "Hanako:false:")?;
        Ok(())
    }

    #[test]
    fn form_data_get_all_length_and_append_work() -> Result<()> {
        let html = r#"
        <form id='f'>
          <input id='t1' name='tag' value='A'>
          <input id='t2' name='tag' value='B'>
        </form>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const fd = new FormData(document.getElementById('f'));
            fd.append('tag', 'C');
            fd.append('other', 123);
            document.getElementById('result').textContent =
              fd.get('tag') + ':' +
              fd.getAll('tag').length + ':' +
              fd.getAll('other').length + ':' +
              fd.get('other');
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "A:3:1:123")?;
        Ok(())
    }

    #[test]
    fn form_data_get_all_length_inline_constructor_works() -> Result<()> {
        let html = r#"
        <form id='f'>
          <input id='t1' name='tag' value='A'>
          <input id='t2' name='tag' value='B'>
        </form>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent =
              new FormData(document.getElementById('f')).getAll('tag').length + ':' +
              new FormData(document.getElementById('f')).getAll('missing').length;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "2:0")?;
        Ok(())
    }

    #[test]
    fn form_data_method_on_non_form_data_variable_returns_runtime_error() -> Result<()> {
        let html = r#"
        <form id='f'>
          <input id='name' name='name' value='Hanako'>
        </form>
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const fd = document.getElementById('f');
            fd.get('name');
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        let err = h
            .click("#btn")
            .expect_err("non-FormData variable should fail on .get()");
        match err {
            Error::ScriptRuntime(msg) => {
                assert!(msg.contains("is not a FormData instance"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
        Ok(())
    }

    #[test]
    fn form_data_append_on_non_form_data_variable_returns_runtime_error() -> Result<()> {
        let html = r#"
        <form id='f'>
          <input id='name' name='name' value='Hanako'>
        </form>
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const fd = document.getElementById('f');
            fd.append('k', 'v');
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        let err = h
            .click("#btn")
            .expect_err("non-FormData variable should fail on .append()");
        match err {
            Error::ScriptRuntime(msg) => {
                assert!(msg.contains("is not a FormData instance"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
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
    fn capture_listeners_fire_in_expected_order() -> Result<()> {
        let html = r#"
        <div id='root'>
          <div id='parent'>
            <button id='btn'>X</button>
          </div>
        </div>
        <p id='result'></p>
        <script>
          document.getElementById('root').addEventListener('click', () => {
            document.getElementById('result').textContent =
              document.getElementById('result').textContent + 'R';
          }, true);
          document.getElementById('parent').addEventListener('click', () => {
            document.getElementById('result').textContent =
              document.getElementById('result').textContent + 'P';
          }, true);
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent =
              document.getElementById('result').textContent + 'C';
          }, true);
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent =
              document.getElementById('result').textContent + 'B';
          });
          document.getElementById('parent').addEventListener('click', () => {
            document.getElementById('result').textContent =
              document.getElementById('result').textContent + 'p';
          });
          document.getElementById('root').addEventListener('click', () => {
            document.getElementById('result').textContent =
              document.getElementById('result').textContent + 'r';
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "RPCBpr")?;
        Ok(())
    }

    #[test]
    fn remove_event_listener_respects_capture_flag() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent =
              document.getElementById('result').textContent + 'C';
          }, true);
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent =
              document.getElementById('result').textContent + 'B';
          });

          document.getElementById('btn').removeEventListener('click', () => {
            document.getElementById('result').textContent =
              document.getElementById('result').textContent + 'C';
          });
          document.getElementById('btn').removeEventListener('click', () => {
            document.getElementById('result').textContent =
              document.getElementById('result').textContent + 'B';
          }, true);
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "CB")?;
        Ok(())
    }

    #[test]
    fn trace_logs_capture_events_when_enabled() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {});
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.enable_trace(true);
        h.click("#btn")?;

        let logs = h.take_trace_logs();
        assert!(logs.iter().any(|line| line.contains("[event] click")));
        assert!(logs.iter().any(|line| line.contains("phase=bubble")));
        assert!(h.take_trace_logs().is_empty());
        Ok(())
    }

    #[test]
    fn trace_logs_collect_when_stderr_output_is_disabled() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {});
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.enable_trace(true);
        h.set_trace_stderr(false);
        h.click("#btn")?;

        let logs = h.take_trace_logs();
        assert!(logs.iter().any(|line| line.contains("[event] click")));
        assert!(logs.iter().any(|line| line.contains("[event] done click")));
        Ok(())
    }

    #[test]
    fn trace_categories_can_disable_timer_logs() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            setTimeout(() => {}, 0);
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.enable_trace(true);
        h.set_trace_stderr(false);
        h.set_trace_timers(false);
        h.click("#btn")?;

        let logs = h.take_trace_logs();
        assert!(logs.iter().any(|line| line.contains("[event] click")));
        assert!(logs.iter().all(|line| !line.contains("[timer]")));
        Ok(())
    }

    #[test]
    fn trace_categories_can_disable_event_logs() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            setTimeout(() => {}, 0);
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.enable_trace(true);
        h.set_trace_stderr(false);
        h.set_trace_events(false);
        h.click("#btn")?;

        let logs = h.take_trace_logs();
        assert!(
            logs.iter()
                .any(|line| line.contains("[timer] schedule timeout id=1"))
        );
        assert!(logs.iter().all(|line| !line.contains("[event]")));
        Ok(())
    }

    #[test]
    fn trace_logs_are_empty_when_trace_is_disabled() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {});
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        assert!(h.take_trace_logs().is_empty());
        Ok(())
    }

    #[test]
    fn trace_logs_capture_timer_lifecycle_when_enabled() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            setTimeout(() => {}, 5);
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.enable_trace(true);
        h.click("#btn")?;

        let logs = h.take_trace_logs();
        assert!(
            logs.iter()
                .any(|line| line.contains("[timer] schedule timeout id=1"))
        );
        assert!(logs.iter().any(|line| line.contains("due_at=5")));
        assert!(logs.iter().any(|line| line.contains("delay_ms=5")));

        assert!(h.run_next_timer()?);
        let logs = h.take_trace_logs();
        assert!(logs.iter().any(|line| line.contains("[timer] run id=1")));
        assert!(logs.iter().any(|line| line.contains("now_ms=5")));
        Ok(())
    }

    #[test]
    fn trace_logs_capture_timer_api_summaries() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            setTimeout(() => {}, 5);
            setTimeout(() => {}, 10);
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.enable_trace(true);
        h.set_trace_stderr(false);
        h.click("#btn")?;
        let _ = h.take_trace_logs();

        h.advance_time(5)?;
        let logs = h.take_trace_logs();
        assert!(
            logs.iter()
                .any(|line| line.contains("[timer] advance delta_ms=5 from=0 to=5 ran_due=1"))
        );

        assert_eq!(h.run_due_timers()?, 0);
        let logs = h.take_trace_logs();
        assert!(
            logs.iter()
                .any(|line| line.contains("[timer] run_due now_ms=5 ran=0"))
        );

        h.flush()?;
        let logs = h.take_trace_logs();
        assert!(
            logs.iter()
                .any(|line| line.contains("[timer] flush from=5 to=10 ran=1"))
        );
        Ok(())
    }

    #[test]
    fn trace_log_limit_keeps_latest_entries() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        "#;

        let mut h = Harness::from_html(html)?;
        h.enable_trace(true);
        h.set_trace_log_limit(2)?;
        h.dispatch("#btn", "alpha")?;
        h.dispatch("#btn", "beta")?;
        h.dispatch("#btn", "gamma")?;

        let logs = h.take_trace_logs();
        assert_eq!(logs.len(), 2);
        assert!(logs.iter().any(|line| line.contains("done beta")));
        assert!(logs.iter().any(|line| line.contains("done gamma")));
        assert!(logs.iter().all(|line| !line.contains("done alpha")));
        Ok(())
    }

    #[test]
    fn set_trace_log_limit_rejects_zero() -> Result<()> {
        let html = r#"<button id='btn'>run</button>"#;
        let mut h = Harness::from_html(html)?;
        let err = h
            .set_trace_log_limit(0)
            .expect_err("zero trace log limit should be rejected");
        match err {
            Error::ScriptRuntime(msg) => {
                assert!(msg.contains("set_trace_log_limit requires at least 1 entry"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
        Ok(())
    }

    #[test]
    fn trace_logs_event_done_contains_default_prevented_and_labels() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', (event) => {
            event.preventDefault();
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.enable_trace(true);
        h.click("#btn")?;
        let logs = h.take_trace_logs();
        assert!(logs.iter().any(|line| line.contains("[event] click")));
        assert!(logs.iter().any(|line| line.contains("target=#btn")));
        assert!(
            logs.iter().any(|line| line.contains("[event] done click")
                && line.contains("default_prevented=true"))
        );
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
    fn query_selector_all_node_list_variable_works() -> Result<()> {
        let html = r#"
        <ul>
          <li class='item'>A</li>
          <li class='item'>B</li>
          <li class='item'>C</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const items = document.querySelectorAll('.item');
            const second = items[1].textContent;
            document.getElementById('result').textContent = items.length + ':' + second;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "3:B")?;
        Ok(())
    }

    #[test]
    fn query_selector_all_index_supports_expression() -> Result<()> {
        let html = r#"
        <ul>
          <li class='item'>A</li>
          <li class='item'>B</li>
          <li class='item'>C</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const items = document.querySelectorAll('.item');
            const index = 1;
            const next = items[index + 1].textContent;
            document.getElementById('result').textContent = items[index].textContent + ':' + next;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "B:C")?;
        Ok(())
    }

    #[test]
    fn query_selector_all_list_index_after_reuse_works() -> Result<()> {
        let html = r#"
        <ul>
          <li class='item'>A</li>
          <li class='item'>B</li>
          <li class='item'>C</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const items = document.querySelectorAll('.item');
            const picked = items[2];
            document.getElementById('result').textContent = picked.textContent + ':' + items.length;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "C:3")?;
        Ok(())
    }

    #[test]
    fn get_elements_by_class_name_works() -> Result<()> {
        let html = r#"
        <ul>
          <li id='x' class='item target'>A</li>
          <li id='y' class='item'>B</li>
          <li id='z' class='target'>C</li>
          <li id='w' class='item target'>D</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const items = document.getElementsByClassName('item target');
            document.getElementById('result').textContent = items.length + ':' + items[0].id + ':' + items[1].id;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "2:x:w")?;
        Ok(())
    }

    #[test]
    fn get_elements_by_tag_name_works() -> Result<()> {
        let html = r#"
        <ul>
          <li id='a'>A</li>
          <li id='b'>B</li>
        </ul>
        <section id='s'>
          <li id='c'>C</li>
        </section>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const items = document.getElementsByTagName('li');
            document.getElementById('result').textContent = items.length + ':' + items[2].id;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "3:c")?;
        Ok(())
    }

    #[test]
    fn get_elements_by_name_works() -> Result<()> {
        let html = r#"
        <input id='a' name='target' value='one'>
        <input id='b' name='other' value='other'>
        <input id='c' name='target' value='two'>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const fields = document.getElementsByName('target');
            document.getElementById('result').textContent = fields.length + ':' + fields[1].value;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "2:two")?;
        Ok(())
    }

    #[test]
    fn class_list_add_remove_multiple_arguments_work() -> Result<()> {
        let html = r#"
        <div id='box' class='base'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            box.classList.add('alpha', 'beta', 'gamma');
            box.classList.remove('base', 'gamma');
            document.getElementById('result').textContent = box.className;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "alpha beta")?;
        Ok(())
    }

    #[test]
    fn class_list_for_each_supports_single_arg_and_index() -> Result<()> {
        let html = r#"
        <div id='box' class='red green blue'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let joined = '';
            let indexes = '';
            document.getElementById('box').classList.forEach((name, index) => {
              joined = joined + name;
              indexes = indexes + index;
            });
            document.getElementById('result').textContent = joined + ':' + indexes;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "redgreenblue:012")?;
        Ok(())
    }

    #[test]
    fn element_click_method_from_script_works() -> Result<()> {
        let html = r#"
        <button id='trigger'>click me</button>
        <input id='agree' type='checkbox'>
        <p id='result'></p>
        <script>
          document.getElementById('trigger').addEventListener('click', () => {
            document.getElementById('agree').click();
            document.getElementById('result').textContent =
              (document.getElementById('agree').checked ? 'checked' : 'unchecked');
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#trigger")?;
        h.assert_text("#result", "checked")?;
        h.click("#trigger")?;
        h.assert_text("#result", "unchecked")?;
        Ok(())
    }

    #[test]
    fn element_scroll_into_view_method_from_script_works() -> Result<()> {
        let html = r#"
        <button id='trigger'>scroll target</button>
        <section id='target'></section>
        <p id='result'></p>
        <script>
          document.getElementById('trigger').addEventListener('click', () => {
            document.getElementById('target').scrollIntoView();
            document.getElementById('result').textContent = 'done';
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#trigger")?;
        h.assert_text("#result", "done")?;
        Ok(())
    }

    #[test]
    fn element_scroll_into_view_rejects_arguments() {
        let html = r#"
        <button id='trigger'>target</button>
        <script>
          document.getElementById('trigger').scrollIntoView('smooth');
        </script>
        "#;

        let err = match Harness::from_html(html) {
            Ok(_) => panic!("scrollIntoView should reject arguments"),
            Err(err) => err,
        };

        match err {
            Error::ScriptParse(msg) => {
                assert_eq!(
                    msg,
                    "scrollIntoView takes no arguments: document.getElementById('trigger').scrollIntoView('smooth')"
                );
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn form_submit_method_dispatches_submit_event() -> Result<()> {
        let html = r#"
        <form id='f'>
          <input id='name' value='default'>
        </form>
        <button id='trigger'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('f').addEventListener('submit', (event) => {
            event.preventDefault();
            document.getElementById('result').textContent =
              event.type + ':' + event.isTrusted + ':' + event.currentTarget.id;
          });
          document.getElementById('trigger').addEventListener('click', () => {
            document.getElementById('f').submit();
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#trigger")?;
        h.assert_text("#result", "submit:true:f")?;
        Ok(())
    }

    #[test]
    fn form_reset_method_dispatches_reset_and_restores_defaults() -> Result<()> {
        let html = r#"
        <form id='f'>
          <input id='name' value='default'>
          <input id='agree' type='checkbox' checked>
        </form>
        <button id='trigger'>run</button>
        <p id='result'></p>
        <script>
          let marker = '';
          document.getElementById('f').addEventListener('reset', () => {
            marker = marker + 'reset';
          });
          document.getElementById('trigger').addEventListener('click', () => {
            document.getElementById('name').value = 'changed';
            document.getElementById('agree').checked = false;
            document.getElementById('f').reset();
            document.getElementById('result').textContent =
              marker + ':' +
              document.getElementById('name').value + ':' +
              (document.getElementById('agree').checked ? 'on' : 'off');
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#trigger")?;
        h.assert_text("#result", "reset:default:on")?;
        Ok(())
    }

    #[test]
    fn element_matches_method_works() -> Result<()> {
        let html = r#"
        <div id='container'>
          <button id='target' class='item primary'></button>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const direct = document.getElementById('target').matches('#target.item');
            const byTag = document.getElementById('target').matches('button');
            const bySelectorMismatch = document.getElementById('target').matches('.secondary');
            document.getElementById('result').textContent =
              direct + ':' + byTag + ':' + bySelectorMismatch;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "true:true:false")?;
        Ok(())
    }

    #[test]
    fn element_closest_method_works() -> Result<()> {
        let html = r#"
        <div id='root'>
          <section id='scope'>
            <div id='container'>
              <button id='btn'>run</button>
            </div>
          </section>
        </div>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const scoped = document.getElementById('btn').closest('section');
            const selfMatch = document.getElementById('btn').closest('#btn');
            document.getElementById('result').textContent =
              scoped.id + ':' + selfMatch.id;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "scope:btn")?;
        Ok(())
    }

    #[test]
    fn element_closest_method_returns_null_when_not_found() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const matched = document.getElementById('btn').closest('section');
            document.getElementById('result').textContent = matched ? 'found' : 'missing';
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "missing")?;
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
    fn parse_for_each_callback_accepts_arrow_expression_body() -> Result<()> {
        let (item_var, index_var, body) = parse_for_each_callback("item => 1")?;
        assert_eq!(item_var, "item");
        assert!(index_var.is_none());
        assert_eq!(body.len(), 1);
        match body.first().expect("callback body should include one statement") {
            Stmt::Expr(Expr::Number(value)) => assert_eq!(*value, 1),
            other => panic!("unexpected callback body stmt: {other:?}"),
        }
        Ok(())
    }

    #[test]
    fn listener_arrow_expression_callback_body_executes() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click',
            () => 1
          );
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.flush()?;
        h.assert_text("#result", "")?;
        Ok(())
    }

    #[test]
    fn for_of_loop_supports_query_selector_all() -> Result<()> {
        let html = r#"
        <ul>
          <li class='item'>A</li>
          <li class='item'>B</li>
          <li class='item'>C</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let output = '';
            for (const item of document.querySelectorAll('.item')) {
              output = output + item.textContent;
            }
            document.getElementById('result').textContent = output;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "ABC")?;
        Ok(())
    }

    #[test]
    fn for_in_loop_supports_query_selector_all_indexes() -> Result<()> {
        let html = r#"
        <ul>
          <li class='item'>A</li>
          <li class='item'>B</li>
          <li class='item'>C</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let output = '';
            for (let index in document.querySelectorAll('.item')) {
              output = output + index + ',';
            }
            document.getElementById('result').textContent = output;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "0,1,2,")?;
        Ok(())
    }

    #[test]
    fn for_loop_supports_break_and_continue() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let out = '';
            for (let i = 0; i < 5; i = i + 1) {
              if (i === 0) {
                continue;
              }
              if (i === 3) {
                break;
              }
              out = out + i;
            }
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "12")?;
        Ok(())
    }

    #[test]
    fn while_loop_supports_break_and_continue() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let out = '';
            let i = 0;
            while (i < 5) {
              i = i + 1;
              if (i === 1) {
                continue;
              }
              if (i === 4) {
                break;
              }
              out = out + i;
            }
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "23")?;
        Ok(())
    }

    #[test]
    fn do_while_executes_at_least_once() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let count = 0;
            do {
              count = count + 1;
            } while (false);
            document.getElementById('result').textContent = count;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "1")?;
        Ok(())
    }

    #[test]
    fn do_while_loop_supports_break_and_continue() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let i = 0;
            let out = '';
            do {
              i = i + 1;
              if (i === 1) {
                continue;
              }
              if (i === 4) {
                break;
              }
              out = out + i;
            } while (i < 5);
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "23")?;
        Ok(())
    }

    #[test]
    fn foreach_supports_break_and_continue() -> Result<()> {
        let html = r#"
        <ul>
          <li class='item'>A</li>
          <li class='item'>B</li>
          <li class='item'>C</li>
          <li class='item'>D</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let out = '';
            document.querySelectorAll('.item').forEach((item, idx) => {
              if (idx === 0) {
                continue;
              }
              if (idx === 2) {
                break;
              }
              out = out + idx;
            });
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "1")?;
        Ok(())
    }

    #[test]
    fn for_in_loop_supports_break_and_continue() -> Result<()> {
        let html = r#"
        <ul>
          <li class='item'>A</li>
          <li class='item'>B</li>
          <li class='item'>C</li>
          <li class='item'>D</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let out = '';
            for (let index in document.querySelectorAll('.item')) {
              if (index === 1) {
                continue;
              }
              if (index === 3) {
                break;
              }
              out = out + index;
            }
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "02")?;
        Ok(())
    }

    #[test]
    fn for_of_loop_supports_break_and_continue() -> Result<()> {
        let html = r#"
        <ul>
          <li class='item'>A</li>
          <li class='item'>B</li>
          <li class='item'>C</li>
          <li class='item'>D</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let out = '';
            for (const item of document.querySelectorAll('.item')) {
              if (item.textContent === 'B') {
                continue;
              }
              if (item.textContent === 'D') {
                break;
              }
              out = out + item.textContent;
            }
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "AC")?;
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
    fn if_block_and_following_statement_without_semicolon_are_split() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let text = '';
            if (true) {
              text = 'A';
            }
            text += 'B';
            document.getElementById('result').textContent = text;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "AB")?;
        Ok(())
    }

    #[test]
    fn while_block_and_following_statement_without_semicolon_are_split() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let count = 0;
            let n = 0;
            while (n < 2) {
              count = count + 1;
              n = n + 1;
            }
            count = count + 10;
            document.getElementById('result').textContent = count;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "12")?;
        Ok(())
    }

    #[test]
    fn for_block_and_following_statement_without_semicolon_are_split() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let sum = 0;
            for (let i = 0; i < 3; i = i + 1) {
              sum = sum + i;
            } sum = sum + 10;
            document.getElementById('result').textContent = sum;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "13")?;
        Ok(())
    }

    #[test]
    fn if_block_and_following_statement_without_space_are_split() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let text = '';
            if (true) {
              text = 'A';
            } if (true) {
              text = text + 'B';
            }
            document.getElementById('result').textContent = text;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "AB")?;
        Ok(())
    }

    #[test]
    fn for_loop_post_increment_with_function_callback_works() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', function() {
            let sum = 0;
            for (let i = 0; i < 3; i++) {
              sum = sum + i;
            }
            document.getElementById('result').textContent = sum;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "3")?;
        Ok(())
    }

    #[test]
    fn promise_then_function_callback_runs_as_microtask() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', function() {
            const result = document.getElementById('result');
            result.textContent = 'A';
            Promise.resolve().then(function() {
              result.textContent = result.textContent + 'P';
            });
            setTimeout(function() {
              result.textContent = result.textContent + 'T';
            }, 0);
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "AP")?;
        h.flush()?;
        h.assert_text("#result", "APT")?;
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
    fn disabled_property_read_write_works() -> Result<()> {
        let html = r#"
        <input id='name' value='init'>
        <button id='toggle'>toggle-disabled</button>
        <button id='enable'>enable</button>
        <p id='result'></p>
        <script>
          document.getElementById('toggle').addEventListener('click', () => {
            document.getElementById('name').disabled = true;
            document.getElementById('result').textContent =
              document.getElementById('name').disabled + ':' +
              document.getElementById('name').getAttribute('disabled');
          });
          document.getElementById('enable').addEventListener('click', () => {
            document.getElementById('name').disabled = false;
            document.getElementById('result').textContent =
              document.getElementById('name').disabled + ':' +
              document.getElementById('name').getAttribute('disabled');
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#toggle")?;
        h.assert_text("#result", "true:true")?;
        h.click("#enable")?;
        h.assert_text("#result", "false:")?;
        Ok(())
    }

    #[test]
    fn readonly_property_read_write_and_type_text_is_ignored() -> Result<()> {
        let html = r#"
        <input id='name' value='init' readonly>
        <button id='make-editable'>editable</button>
        <button id='confirm'>confirm</button>
        <p id='result'></p>
        <script>
          document.getElementById('make-editable').addEventListener('click', () => {
            document.getElementById('name').readonly = false;
            document.getElementById('result').textContent = document.getElementById('name').readonly;
          });
          document.getElementById('confirm').addEventListener('click', () => {
            document.getElementById('result').textContent =
              document.getElementById('name').readonly + ':' +
              document.getElementById('name').value;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.type_text("#name", "changed")?;
        h.assert_value("#name", "init")?;
        h.click("#make-editable")?;
        h.type_text("#name", "changed")?;
        h.assert_value("#name", "changed")?;
        h.click("#confirm")?;
        h.assert_text("#result", "false:changed")?;
        Ok(())
    }

    #[test]
    fn required_property_read_write_works() -> Result<()> {
        let html = r#"
        <input id='name' required>
        <button id='unset'>unset</button>
        <button id='set'>set</button>
        <p id='result'></p>
        <script>
          document.getElementById('set').addEventListener('click', () => {
            document.getElementById('name').required = true;
            document.getElementById('result').textContent = document.getElementById('name').required;
          });
          document.getElementById('unset').addEventListener('click', () => {
            document.getElementById('name').required = false;
            document.getElementById('result').textContent = document.getElementById('name').required;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#unset")?;
        h.assert_text("#result", "false")?;
        h.click("#set")?;
        h.assert_text("#result", "true")?;
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
    fn offset_and_scroll_properties_are_read_only_and_queryable() -> Result<()> {
        let html = r#"
        <div id='box' style='width: 120px; height: 90px;'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            document.getElementById('result').textContent =
              box.offsetWidth + ':' + box.offsetHeight + ':' +
              box.offsetTop + ':' + box.offsetLeft + ':' +
              box.scrollWidth + ':' + box.scrollHeight + ':' +
              box.scrollTop + ':' + box.scrollLeft;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "0:0:0:0:0:0:0:0")?;
        Ok(())
    }

    #[test]
    fn offset_property_assignment_is_rejected() -> Result<()> {
        let html = r#"
        <div id='box'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('box').scrollTop = 10;
            document.getElementById('box').offsetWidth = 100;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        let err = h
            .click("#btn")
            .expect_err("scrollTop/offsetWidth assignment should fail");
        assert!(format!("{err}").contains("is read-only"));
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
    fn focus_and_blur_update_active_element_and_events() -> Result<()> {
        let html = r#"
        <input id='a'>
        <input id='b'>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          const a = document.getElementById('a');
          const b = document.getElementById('b');
          let order = '';

          a.addEventListener('focus', () => {
            order += 'aF';
          });
          a.addEventListener('blur', () => {
            order += 'aB';
          });
          b.addEventListener('focus', () => {
            order += 'bF';
          });
          b.addEventListener('blur', () => {
            order += 'bB';
          });

          document.getElementById('btn').addEventListener('click', () => {
            a.focus();
            b.focus();
            b.blur();
            document.getElementById('result').textContent =
              order + ':' + (document.activeElement === null ? 'none' : 'active');
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "aFaBbFbB:none")?;
        Ok(())
    }

    #[test]
    fn focus_in_and_focus_out_events_are_dispatched() -> Result<()> {
        let html = r#"
        <input id='a'>
        <input id='b'>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          const a = document.getElementById('a');
          const b = document.getElementById('b');
          let order = '';

          a.addEventListener('focusin', () => {
            order += 'aI';
          });
          a.addEventListener('focus', () => {
            order += 'aF';
          });
          a.addEventListener('focusout', () => {
            order += 'aO';
          });
          a.addEventListener('blur', () => {
            order += 'aB';
          });

          b.addEventListener('focusin', () => {
            order += 'bI';
          });
          b.addEventListener('focus', () => {
            order += 'bF';
          });
          b.addEventListener('focusout', () => {
            order += 'bO';
          });
          b.addEventListener('blur', () => {
            order += 'bB';
          });

          document.getElementById('btn').addEventListener('click', () => {
            a.focus();
            b.focus();
            b.blur();
            document.getElementById('result').textContent =
              order + ':' + (document.activeElement === null ? 'none' : 'active');
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "aIaFaOaBbIbFbObB:none")?;
        Ok(())
    }

    #[test]
    fn focus_skips_disabled_element() -> Result<()> {
        let html = r#"
        <input id='name' disabled>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('name').focus();
            document.getElementById('result').textContent = document.activeElement ? 'has' : 'none';
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "none")?;
        Ok(())
    }

    #[test]
    fn selector_focus_and_focus_within_runtime() -> Result<()> {
        let html = r#"
        <div id='scope'>
          <input id='child'>
        </div>
        <input id='outside'>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const child = document.getElementById('child');
            const outside = document.getElementById('outside');
            child.focus();
            const before = document.querySelector('input:focus').id + ':' +
              (document.querySelectorAll('#scope:focus-within').length ? 'yes' : 'no');
            outside.focus();
            const after = document.querySelector('input:focus').id + ':' +
              (document.querySelectorAll('#scope:focus-within').length ? 'yes' : 'no');
            document.getElementById('result').textContent = before + ':' + after;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "child:yes:outside:no")?;
        Ok(())
    }

    #[test]
    fn selector_active_is_set_during_click_and_cleared_after() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const during = document.querySelectorAll('#btn:active').length ? 'yes' : 'no';
            setTimeout(() => {
              const after = document.querySelectorAll('#btn:active').length ? 'yes' : 'no';
              document.getElementById('result').textContent = during + ':' + after;
            }, 0);
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.advance_time(0)?;
        h.assert_text("#result", "yes:no")?;
        Ok(())
    }

    #[test]
    fn active_element_assignment_is_read_only() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.activeElement = null;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        let err = h
            .click("#btn")
            .expect_err("activeElement should be read-only");
        match err {
            Error::ScriptRuntime(msg) => {
                assert!(msg.contains("read-only"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
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
    fn get_computed_style_property_value_works() -> Result<()> {
        let html = r#"
        <div id='box' style='color: blue; background-color: transparent;'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            box.style.color = 'red';
            const color = getComputedStyle(box).getPropertyValue('color');
            const missing = getComputedStyle(box).getPropertyValue('padding-top');
            document.getElementById('result').textContent = color + ':' + missing;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "red:")?;
        Ok(())
    }

    #[test]
    fn style_parser_supports_quoted_colon_and_semicolon() -> Result<()> {
        let html = r#"
        <div id='box' style='content: "a:b;c"; color: blue;'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            document.getElementById('result').textContent =
              box.style.content + ':' + box.style.color + ':' + box.getAttribute('style');
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text(
            "#result",
            "\"a:b;c\":blue:content: \"a:b;c\"; color: blue;",
        )?;
        Ok(())
    }

    #[test]
    fn style_parser_supports_parentheses_values() -> Result<()> {
        let html = r#"
        <div id='box' style='background-image: url("a;b:c"); font-family: Arial, sans-serif;'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            document.getElementById('result').textContent =
              box.style.backgroundImage + ':' + box.style.fontFamily + ':' + box.getAttribute('style');
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text(
            "#result",
            "url(\"a;b:c\"):Arial, sans-serif:background-image: url(\"a;b:c\"); font-family: Arial, sans-serif;",
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
    fn event_trusted_and_target_subproperties_are_accessible() -> Result<()> {
        let html = r#"
        <div id='root'>
          <button id='btn' name='target-name'>run</button>
        </div>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', (event) => {
            event.preventDefault();
            document.getElementById('result').textContent =
              event.isTrusted + ':' +
              event.target.name + ':' +
              event.currentTarget.name;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "true:target-name:target-name")?;
        Ok(())
    }

    #[test]
    fn event_bubbles_and_cancelable_properties_are_available() -> Result<()> {
        let html = r#"
        <div id='root'>
          <button id='btn' name='target-name'>run</button>
        </div>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', (event) => {
            document.getElementById('result').textContent =
              event.bubbles + ':' + event.cancelable + ':' + event.isTrusted;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "true:true:true")?;
        Ok(())
    }

    #[test]
    fn dispatch_event_origin_is_untrusted_and_supports_event_methods() -> Result<()> {
        let html = r#"
        <div id='root'>
          <div id='box'></div>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('root').addEventListener('custom', (event) => {
            document.getElementById('result').textContent = 'root:' + event.target.id;
          });
          document.getElementById('box').addEventListener('custom', (event) => {
            event.preventDefault();
            event.stopPropagation();
            document.getElementById('result').textContent =
              event.isTrusted + ':' + event.defaultPrevented;
          });
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('box').dispatchEvent('custom');
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "false:true")?;
        Ok(())
    }

    #[test]
    fn event_default_prevented_property_reflects_prevent_default() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', (event) => {
            document.getElementById('result').textContent =
              event.defaultPrevented + ',';
            event.preventDefault();
            document.getElementById('result').textContent =
              document.getElementById('result').textContent + event.defaultPrevented;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "false,true")?;
        Ok(())
    }

    #[test]
    fn event_phase_and_timestamp_are_available_in_handler() -> Result<()> {
        let html = r#"
        <div id='root'>
          <button id='btn'>run</button>
        </div>
        <p id='result'></p>
        <script>
          let phases = '';
          document.getElementById('root').addEventListener('click', (event) => {
            phases = phases + (phases === '' ? '' : ',') + event.eventPhase + ':' + event.timeStamp;
          }, true);
          document.getElementById('btn').addEventListener('click', (event) => {
            phases = phases + ',' + event.eventPhase + ':' + event.timeStamp;
          }, true);
          document.getElementById('btn').addEventListener('click', (event) => {
            phases = phases + ',' + event.eventPhase + ':' + event.timeStamp;
          });
          document.getElementById('root').addEventListener('click', (event) => {
            phases = phases + ',' + event.eventPhase + ':' + event.timeStamp;
            document.getElementById('result').textContent = phases;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "1:0,2:0,2:0,3:0")?;
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
    fn set_timeout_runs_on_flush_and_captures_env() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            result.textContent = 'A';
            setTimeout(() => {
              result.textContent = result.textContent + 'B';
            }, 0);
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "A")?;
        h.flush()?;
        h.assert_text("#result", "AB")?;
        Ok(())
    }

    #[test]
    fn timer_arguments_support_additional_parameters_and_comments() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            // comment: schedule timer with extra arg and inline delay comment
            setTimeout((message) => {
              document.getElementById('result').textContent = message;
            }, 5, 'ok');
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "")?;
        h.advance_time(4)?;
        h.assert_text("#result", "")?;
        h.advance_time(1)?;
        h.assert_text("#result", "ok")?;
        Ok(())
    }

    #[test]
    fn timer_callback_supports_multiple_parameters() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            setTimeout((first, second, third) => {
              document.getElementById('result').textContent =
                first + ':' + second + ':' + third;
            }, 5, 'A', 'B', 'C');
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "")?;
        h.advance_time(5)?;
        h.assert_text("#result", "A:B:C")?;
        Ok(())
    }

    #[test]
    fn timer_callback_assigns_undefined_for_missing_arguments() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            setTimeout((first, second, third) => {
              document.getElementById('result').textContent =
                first + ':' + second + ':' + third;
            }, 5, 'only');
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "")?;
        h.advance_time(5)?;
        h.assert_text("#result", "only:undefined:undefined")?;
        Ok(())
    }

    #[test]
    fn timer_function_reference_supports_additional_parameters() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          const onTimeout = (value) => {
            document.getElementById('result').textContent = value;
          };
          document.getElementById('btn').addEventListener('click', () => {
            setTimeout(onTimeout, 5, 'ref');
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "")?;
        h.advance_time(5)?;
        h.assert_text("#result", "ref")?;
        Ok(())
    }

    #[test]
    fn timer_interval_function_reference_supports_additional_parameters() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          let count = 0;
          const onTick = (value) => {
            count = count + 1;
            document.getElementById('result').textContent =
              document.getElementById('result').textContent + value;
            if (count === 2) {
              clearInterval(intervalId);
            }
          };
          let intervalId = 0;
          document.getElementById('btn').addEventListener('click', () => {
            intervalId = setInterval(onTick, 5, 'tick');
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "")?;
        h.advance_time(11)?;
        h.assert_text("#result", "ticktick")?;
        h.advance_time(10)?;
        h.assert_text("#result", "ticktick")?;
        Ok(())
    }

    #[test]
    fn timer_interval_supports_multiple_additional_parameters() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          let id = 0;
          document.getElementById('btn').addEventListener('click', () => {
            let tick = 0;
            id = setInterval((value, suffix) => {
              tick = tick + 1;
              document.getElementById('result').textContent =
                document.getElementById('result').textContent + value + suffix;
              if (tick > 2) {
                clearInterval(id);
              }
            }, 0, 'I', '!');
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.flush()?;
        h.assert_text("#result", "I!I!I!")?;
        Ok(())
    }

    #[test]
    fn line_and_block_comments_are_ignored_in_script_parser() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          // top level comment
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent = 'A'; // inline comment
            /* block comment */
            document.getElementById('result').textContent =
              document.getElementById('result').textContent + 'B';
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "AB")?;
        Ok(())
    }

    #[test]
    fn run_due_timers_runs_only_currently_due_tasks() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            setTimeout(() => {
              result.textContent = result.textContent + 'A';
            }, 0);
            setTimeout(() => {
              result.textContent = result.textContent + 'B';
            }, 5);
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        assert_eq!(h.now_ms(), 0);

        let ran = h.run_due_timers()?;
        assert_eq!(ran, 1);
        assert_eq!(h.now_ms(), 0);
        h.assert_text("#result", "A")?;

        let ran = h.run_due_timers()?;
        assert_eq!(ran, 0);
        h.assert_text("#result", "A")?;
        Ok(())
    }

    #[test]
    fn run_due_timers_returns_zero_for_empty_queue() -> Result<()> {
        let html = r#"<button id='btn'>run</button>"#;
        let mut h = Harness::from_html(html)?;
        assert_eq!(h.run_due_timers()?, 0);
        Ok(())
    }

    #[test]
    fn clear_timer_cancels_specific_pending_timer() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            setTimeout(() => {
              result.textContent = result.textContent + 'A';
            }, 5);
            setTimeout(() => {
              result.textContent = result.textContent + 'B';
            }, 0);
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        assert!(h.clear_timer(1));
        assert!(!h.clear_timer(1));
        assert!(!h.clear_timer(999));

        h.advance_time(0)?;
        h.assert_text("#result", "B")?;
        h.advance_time(10)?;
        h.assert_text("#result", "B")?;
        Ok(())
    }

    #[test]
    fn clear_all_timers_empties_pending_queue() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            setTimeout(() => {
              result.textContent = 'A';
            }, 0);
            setInterval(() => {
              result.textContent = 'B';
            }, 5);
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        assert_eq!(h.pending_timers().len(), 2);
        assert_eq!(h.clear_all_timers(), 2);
        assert!(h.pending_timers().is_empty());
        h.flush()?;
        h.assert_text("#result", "")?;
        Ok(())
    }

    #[test]
    fn run_next_due_timer_runs_only_one_due_task_without_advancing_clock() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            setTimeout(() => {
              result.textContent = result.textContent + 'A';
            }, 0);
            setTimeout(() => {
              result.textContent = result.textContent + 'B';
            }, 5);
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        assert_eq!(h.now_ms(), 0);

        assert!(h.run_next_due_timer()?);
        assert_eq!(h.now_ms(), 0);
        h.assert_text("#result", "A")?;

        assert!(!h.run_next_due_timer()?);
        assert_eq!(h.now_ms(), 0);
        h.assert_text("#result", "A")?;
        Ok(())
    }

    #[test]
    fn run_next_due_timer_returns_false_for_empty_queue() -> Result<()> {
        let html = r#"<button id='btn'>run</button>"#;
        let mut h = Harness::from_html(html)?;
        assert!(!h.run_next_due_timer()?);
        Ok(())
    }

    #[test]
    fn pending_timers_returns_due_ordered_snapshot() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            setTimeout(() => {}, 10);
            setInterval(() => {}, 5);
            setTimeout(() => {}, 0);
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        let timers = h.pending_timers();
        assert_eq!(
            timers,
            vec![
                PendingTimer {
                    id: 3,
                    due_at: 0,
                    order: 2,
                    interval_ms: None,
                },
                PendingTimer {
                    id: 2,
                    due_at: 5,
                    order: 1,
                    interval_ms: Some(5),
                },
                PendingTimer {
                    id: 1,
                    due_at: 10,
                    order: 0,
                    interval_ms: None,
                },
            ]
        );
        Ok(())
    }

    #[test]
    fn pending_timers_reflects_advance_time_execution() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            setInterval(() => {}, 5);
            setTimeout(() => {}, 7);
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.advance_time(5)?;

        let timers = h.pending_timers();
        assert_eq!(
            timers,
            vec![
                PendingTimer {
                    id: 2,
                    due_at: 7,
                    order: 1,
                    interval_ms: None,
                },
                PendingTimer {
                    id: 1,
                    due_at: 10,
                    order: 2,
                    interval_ms: Some(5),
                },
            ]
        );
        Ok(())
    }

    #[test]
    fn run_next_timer_executes_single_task_in_due_order() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            setTimeout(() => {
              result.textContent = result.textContent + 'A';
            }, 10);
            setTimeout(() => {
              result.textContent = result.textContent + 'B';
            }, 0);
            setTimeout(() => {
              result.textContent = result.textContent + 'C';
            }, 10);
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        assert_eq!(h.now_ms(), 0);

        assert!(h.run_next_timer()?);
        assert_eq!(h.now_ms(), 0);
        h.assert_text("#result", "B")?;

        assert!(h.run_next_timer()?);
        assert_eq!(h.now_ms(), 10);
        h.assert_text("#result", "BA")?;

        assert!(h.run_next_timer()?);
        assert_eq!(h.now_ms(), 10);
        h.assert_text("#result", "BAC")?;

        assert!(!h.run_next_timer()?);
        assert_eq!(h.now_ms(), 10);
        Ok(())
    }

    #[test]
    fn advance_time_to_runs_due_timers_until_target() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            setTimeout(() => {
              result.textContent = result.textContent + 'A';
            }, 5);
            setTimeout(() => {
              result.textContent = result.textContent + 'B';
            }, 10);
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.advance_time_to(7)?;
        assert_eq!(h.now_ms(), 7);
        h.assert_text("#result", "A")?;

        h.advance_time_to(10)?;
        assert_eq!(h.now_ms(), 10);
        h.assert_text("#result", "AB")?;

        h.advance_time_to(10)?;
        assert_eq!(h.now_ms(), 10);
        h.assert_text("#result", "AB")?;
        Ok(())
    }

    #[test]
    fn advance_time_to_rejects_past_target() -> Result<()> {
        let html = r#"<button id='btn'>run</button>"#;
        let mut h = Harness::from_html(html)?;
        h.advance_time(3)?;
        let err = h
            .advance_time_to(2)
            .expect_err("advance_time_to with past target should fail");
        match err {
            Error::ScriptRuntime(msg) => {
                assert!(msg.contains("advance_time_to requires target >= now_ms"));
                assert!(msg.contains("target=2"));
                assert!(msg.contains("now_ms=3"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
        Ok(())
    }

    #[test]
    fn set_timeout_respects_delay_order_and_nested_queueing() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            setTimeout(() => {
              result.textContent = result.textContent + '1';
            }, 10);
            setTimeout(() => {
              result.textContent = result.textContent + '0';
              setTimeout(() => {
                result.textContent = result.textContent + 'N';
              });
            }, 0);
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "")?;
        h.flush()?;
        h.assert_text("#result", "0N1")?;
        Ok(())
    }

    #[test]
    fn queue_microtask_runs_after_synchronous_task_body() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            result.textContent = 'A';
            queueMicrotask(() => {
              result.textContent = result.textContent + 'B';
            });
            result.textContent = result.textContent + 'C';
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "ACB")?;
        Ok(())
    }

    #[test]
    fn promise_then_microtask_runs_before_next_timer() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            result.textContent = 'A';
            Promise.resolve().then(() => {
              result.textContent = result.textContent + 'P';
            });
            setTimeout(() => {
              result.textContent = result.textContent + 'T';
            }, 0);
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "AP")?;
        h.flush()?;
        h.assert_text("#result", "APT")?;
        Ok(())
    }

    #[test]
    fn fake_time_advance_controls_timer_execution() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            setTimeout(() => {
              result.textContent = result.textContent + '0';
            }, 0);
            setTimeout(() => {
              result.textContent = result.textContent + '1';
            }, 10);
            setTimeout(() => {
              result.textContent = result.textContent + '2';
            }, 20);
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "")?;
        assert_eq!(h.now_ms(), 0);

        h.advance_time(0)?;
        h.assert_text("#result", "0")?;
        assert_eq!(h.now_ms(), 0);

        h.advance_time(9)?;
        h.assert_text("#result", "0")?;
        assert_eq!(h.now_ms(), 9);

        h.advance_time(1)?;
        h.assert_text("#result", "01")?;
        assert_eq!(h.now_ms(), 10);

        h.advance_time(10)?;
        h.assert_text("#result", "012")?;
        assert_eq!(h.now_ms(), 20);
        Ok(())
    }

    #[test]
    fn fake_time_advance_runs_interval_ticks_by_due_time() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            const id = setInterval(() => {
              result.textContent = result.textContent + 'I';
              if (result.textContent === 'III') clearInterval(id);
            }, 5);
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "")?;

        h.advance_time(4)?;
        h.assert_text("#result", "")?;

        h.advance_time(1)?;
        h.assert_text("#result", "I")?;

        h.advance_time(10)?;
        h.assert_text("#result", "III")?;

        h.advance_time(100)?;
        h.assert_text("#result", "III")?;
        Ok(())
    }

    #[test]
    fn date_now_uses_fake_clock_for_handlers_and_timers() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            result.textContent = Date.now() + ':';
            setTimeout(() => {
              result.textContent = result.textContent + Date.now();
            }, 10);
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.advance_time(7)?;
        h.click("#btn")?;
        h.assert_text("#result", "7:")?;

        h.advance_time(9)?;
        h.assert_text("#result", "7:")?;

        h.advance_time(1)?;
        h.assert_text("#result", "7:17")?;
        Ok(())
    }

    #[test]
    fn date_now_with_flush_advances_to_timer_due_time() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            result.textContent = Date.now();
            setTimeout(() => {
              result.textContent = result.textContent + ':' + Date.now();
            }, 25);
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "0")?;
        h.flush()?;
        h.assert_text("#result", "0:25")?;
        assert_eq!(h.now_ms(), 25);
        Ok(())
    }

    #[test]
    fn date_constructor_and_static_methods_work() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const nowDate = new Date();
            const fromNumber = new Date(1000);
            const parsed = Date.parse('1970-01-01T00:00:02Z');
            const utc = Date.UTC(1970, 0, 1, 0, 0, 3);
            const parsedViaWindow = window.Date.parse('1970-01-01');
            document.getElementById('result').textContent =
              nowDate.getTime() + ':' + fromNumber.getTime() + ':' +
              parsed + ':' + utc + ':' + parsedViaWindow;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.advance_time(42)?;
        h.click("#btn")?;
        h.assert_text("#result", "42:1000:2000:3000:0")?;
        Ok(())
    }

    #[test]
    fn date_instance_methods_and_set_time_work() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const d = new Date('2024-03-05T01:02:03Z');
            const y = d.getFullYear();
            const m = d.getMonth();
            const day = d.getDate();
            const h = d.getHours();
            const min = d.getMinutes();
            const s = d.getSeconds();
            const iso = d.toISOString();
            const updated = d.setTime(Date.UTC(1970, 0, 2, 3, 4, 5));
            const iso2 = d.toISOString();
            document.getElementById('result').textContent =
              y + ':' + m + ':' + day + ':' + h + ':' + min + ':' + s +
              '|' + iso + '|' + updated + '|' + iso2;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text(
            "#result",
            "2024:2:5:1:2:3|2024-03-05T01:02:03.000Z|97445000|1970-01-02T03:04:05.000Z",
        )?;
        Ok(())
    }

    #[test]
    fn date_parse_invalid_input_returns_nan_and_utc_normalizes_overflow() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const parsedValue = Date.parse('invalid-date');
            const isInvalid = isNaN(parsedValue);
            const ts = Date.UTC(2020, 12, 1, 25, 61, 61);
            const normalizedDate = new Date(ts);
            const normalized = normalizedDate.toISOString();
            document.getElementById('result').textContent =
              isInvalid + ':' + normalized;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "true:2021-01-02T02:02:01.000Z")?;
        Ok(())
    }

    #[test]
    fn date_method_arity_errors_have_stable_messages() {
        let cases = [
            (
                "<script>new Date(1, 2);</script>",
                "new Date supports zero or one argument",
            ),
            (
                "<script>Date.parse();</script>",
                "Date.parse requires exactly one argument",
            ),
            (
                "<script>Date.UTC(1970);</script>",
                "Date.UTC requires between 2 and 6 arguments",
            ),
            (
                "<script>Date.UTC(1970, , 1);</script>",
                "Date.UTC argument cannot be empty",
            ),
            (
                "<script>const d = new Date(); d.getTime(1);</script>",
                "getTime does not take arguments",
            ),
            (
                "<script>const d = new Date(); d.setTime();</script>",
                "setTime requires exactly one argument",
            ),
            (
                "<script>const d = new Date(); d.toISOString(1);</script>",
                "toISOString does not take arguments",
            ),
        ];

        for (html, expected) in cases {
            let err = Harness::from_html(html).expect_err("script should fail to parse");
            match err {
                Error::ScriptParse(msg) => assert!(
                    msg.contains(expected),
                    "expected '{expected}' in '{msg}'"
                ),
                other => panic!("unexpected error: {other:?}"),
            }
        }
    }

    #[test]
    fn math_random_is_deterministic_with_seed() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent =
              Math.random() + ':' + Math.random() + ':' + Math.random();
          });
        </script>
        "#;

        let mut h1 = Harness::from_html(html)?;
        let mut h2 = Harness::from_html(html)?;
        h1.set_random_seed(12345);
        h2.set_random_seed(12345);

        h1.click("#btn")?;
        h2.click("#btn")?;

        let out1 = h1.dump_dom("#result")?;
        let out2 = h2.dump_dom("#result")?;
        assert_eq!(out1, out2);
        Ok(())
    }

    #[test]
    fn math_random_returns_unit_interval() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const r = Math.random();
            if (r >= 0 && r < 1) document.getElementById('result').textContent = 'ok';
            else document.getElementById('result').textContent = 'ng';
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.set_random_seed(42);
        h.click("#btn")?;
        h.assert_text("#result", "ok")?;
        Ok(())
    }

    #[test]
    fn decimal_numeric_literals_work_in_comparisons_and_assignment() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = 0.5;
            const b = 1.0;
            if (a < b && a === 0.5 && b >= 1)
              document.getElementById('result').textContent = a;
            else
              document.getElementById('result').textContent = 'ng';
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "0.5")?;
        Ok(())
    }

    #[test]
    fn multiplication_and_division_work_for_numbers() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = 6 * 7;
            const b = 5 / 2;
            document.getElementById('result').textContent = a + ':' + b;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "42:2.5")?;
        Ok(())
    }

    #[test]
    fn subtraction_and_unary_minus_work_for_numbers() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = 10 - 3;
            const b = -2;
            const c = 1 - -2;
            document.getElementById('result').textContent = a + ':' + b + ':' + c;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "7:-2:3")?;
        Ok(())
    }

    #[test]
    fn addition_supports_numeric_and_string_left_fold() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = 1 + 2;
            const b = 1 + 2 + 'x';
            const c = 1 + '2' + 3;
            document.getElementById('result').textContent = a + ':' + b + ':' + c;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "3:3x:123")?;
        Ok(())
    }

    #[test]
    fn timer_delay_accepts_arithmetic_expression() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            setTimeout(() => {
              document.getElementById('result').textContent = 'ok';
            }, 5 * 2);
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "")?;
        h.advance_time(9)?;
        h.assert_text("#result", "")?;
        h.advance_time(1)?;
        h.assert_text("#result", "ok")?;
        Ok(())
    }

    #[test]
    fn timer_delay_accepts_addition_expression() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            setTimeout(() => {
              document.getElementById('result').textContent = 'ok';
            }, 5 + 5);
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "")?;
        h.advance_time(10)?;
        h.assert_text("#result", "ok")?;
        Ok(())
    }

    #[test]
    fn timer_delay_accepts_subtraction_expression() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            setTimeout(() => {
              document.getElementById('result').textContent = 'ok';
            }, 15 - 5);
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "")?;
        h.advance_time(10)?;
        h.assert_text("#result", "ok")?;
        Ok(())
    }

    #[test]
    fn timer_arrow_expression_callback_executes() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            setTimeout(
              () => setTimeout(() => {
                document.getElementById('result').textContent = 'ok';
              }, 0),
              5
            );
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "")?;
        h.advance_time(5)?;
        h.assert_text("#result", "ok")?;
        Ok(())
    }

    #[test]
    fn math_random_seed_reset_repeats_sequence() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent =
              Math.random() + ':' + Math.random();
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.set_random_seed(7);
        h.click("#btn")?;
        let first = h.dump_dom("#result")?;

        h.set_random_seed(7);
        h.click("#btn")?;
        let second = h.dump_dom("#result")?;

        assert_eq!(first, second);
        Ok(())
    }

    #[test]
    fn clear_timeout_cancels_task_and_set_timeout_returns_ids() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            const first = setTimeout(() => {
              result.textContent = result.textContent + 'A';
            }, 5);
            const second = setTimeout(() => {
              result.textContent = result.textContent + 'B';
            }, 0);
            clearTimeout(first);
            result.textContent = first + ':' + second + ':';
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "1:2:")?;
        h.flush()?;
        h.assert_text("#result", "1:2:B")?;
        Ok(())
    }

    #[test]
    fn clear_timeout_unknown_id_is_ignored() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            clearTimeout(999);
            setTimeout(() => {
              document.getElementById('result').textContent = 'ok';
            }, 0);
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "")?;
        h.flush()?;
        h.assert_text("#result", "ok")?;
        Ok(())
    }

    #[test]
    fn script_extractor_ignores_script_like_strings() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          const marker = "</script>";
          const htmlLike = "<script>not real</script>";
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent = marker + '|' + htmlLike;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "</script>|<script>not real</script>")?;
        Ok(())
    }

    #[test]
    fn set_interval_repeats_and_clear_interval_stops_requeue() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            const id = setInterval(() => {
              result.textContent = result.textContent + 'I';
              if (result.textContent === '1:III') clearInterval(id);
            }, 0);
            result.textContent = id + ':';
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "1:")?;
        h.flush()?;
        h.assert_text("#result", "1:III")?;
        h.flush()?;
        h.assert_text("#result", "1:III")?;
        Ok(())
    }

    #[test]
    fn clear_timeout_can_cancel_interval_id() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            const id = setInterval(() => {
              result.textContent = result.textContent + 'X';
            }, 0);
            clearTimeout(id);
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.flush()?;
        h.assert_text("#result", "")?;
        Ok(())
    }

    #[test]
    fn flush_step_limit_error_contains_timer_diagnostics() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            setInterval(() => {}, 0);
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        let err = h
            .flush()
            .expect_err("flush should fail on uncleared interval");
        match err {
            Error::ScriptRuntime(msg) => {
                assert!(msg.contains("flush exceeded max task steps"));
                assert!(msg.contains("limit=10000"));
                assert!(msg.contains("pending_tasks="));
                assert!(msg.contains("next_task=id=1"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
        Ok(())
    }

    #[test]
    fn timer_step_limit_can_be_configured() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            setInterval(() => {}, 0);
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.set_timer_step_limit(3)?;
        h.click("#btn")?;
        let err = h
            .flush()
            .expect_err("flush should fail with configured small step limit");
        match err {
            Error::ScriptRuntime(msg) => {
                assert!(msg.contains("limit=3"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
        Ok(())
    }

    #[test]
    fn timer_step_limit_rejects_zero() -> Result<()> {
        let html = r#"<button id='btn'>run</button>"#;
        let mut h = Harness::from_html(html)?;
        let err = h
            .set_timer_step_limit(0)
            .expect_err("zero step limit should be rejected");
        match err {
            Error::ScriptRuntime(msg) => {
                assert!(msg.contains("set_timer_step_limit requires at least 1 step"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
        Ok(())
    }

    #[test]
    fn advance_time_step_limit_error_contains_due_limit() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            setInterval(() => {}, 0);
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.set_timer_step_limit(2)?;
        h.click("#btn")?;
        let err = h
            .advance_time(7)
            .expect_err("advance_time should fail with configured small step limit");
        match err {
            Error::ScriptRuntime(msg) => {
                assert!(msg.contains("limit=2"));
                assert!(msg.contains("now_ms=7"));
                assert!(msg.contains("due_limit=7"));
                assert!(msg.contains("next_task=id=1"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
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
    fn create_element_append_and_remove_child_work() -> Result<()> {
        let html = r#"
        <div id='root'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const root = document.getElementById('root');
            const node = document.createElement('span');
            node.id = 'tmp';
            node.textContent = 'X';

            document.getElementById('result').textContent =
              document.querySelectorAll('#tmp').length + ':';
            root.appendChild(node);
            document.getElementById('result').textContent =
              document.getElementById('result').textContent +
              document.querySelectorAll('#tmp').length + ':' +
              document.querySelector('#root>#tmp').textContent;
            root.removeChild(node);
            document.getElementById('result').textContent =
              document.getElementById('result').textContent + ':' +
              document.querySelectorAll('#tmp').length;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "0:1:X:0")?;
        Ok(())
    }

    #[test]
    fn insert_before_inserts_new_node_before_reference() -> Result<()> {
        let html = r#"
        <div id='root'><span id='a'>A</span><span id='c'>C</span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const root = document.getElementById('root');
            const b = document.createElement('span');
            b.id = 'b';
            b.textContent = 'B';
            root.insertBefore(b, document.getElementById('c'));
            document.getElementById('result').textContent =
              root.textContent + ':' + document.querySelector('#root>#b').textContent;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "ABC:B")?;
        Ok(())
    }

    #[test]
    fn insert_before_reorders_existing_child() -> Result<()> {
        let html = r#"
        <div id='root'><span id='a'>A</span><span id='b'>B</span><span id='c'>C</span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const root = document.getElementById('root');
            root.insertBefore(
              document.getElementById('c'),
              document.getElementById('a')
            );
            document.getElementById('result').textContent = root.textContent;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "CAB")?;
        Ok(())
    }

    #[test]
    fn append_alias_adds_child_to_end() -> Result<()> {
        let html = r#"
        <div id='root'><span>A</span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const root = document.getElementById('root');
            const b = document.createElement('span');
            b.id = 'b';
            b.textContent = 'B';
            root.append(b);
            document.getElementById('result').textContent =
              root.textContent + ':' + document.querySelector('#root>#b').textContent;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "AB:B")?;
        Ok(())
    }

    #[test]
    fn prepend_adds_child_to_start() -> Result<()> {
        let html = r#"
        <div id='root'><span id='b'>B</span><span id='c'>C</span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const root = document.getElementById('root');
            const a = document.createElement('span');
            a.id = 'a';
            a.textContent = 'A';
            root.prepend(a);
            document.getElementById('result').textContent =
              root.textContent + ':' + document.querySelector('#root>#a').textContent;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "ABC:A")?;
        Ok(())
    }

    #[test]
    fn before_and_after_insert_relative_to_target() -> Result<()> {
        let html = r#"
        <div id='root'><span id='b'>B</span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const b = document.getElementById('b');
            const a = document.createElement('span');
            a.id = 'a';
            a.textContent = 'A';
            const c = document.createElement('span');
            c.id = 'c';
            c.textContent = 'C';
            b.before(a);
            b.after(c);
            document.getElementById('result').textContent =
              document.getElementById('root').textContent + ':' +
              document.querySelector('#root>#a').textContent + ':' +
              document.querySelector('#root>#c').textContent;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "ABC:A:C")?;
        Ok(())
    }

    #[test]
    fn replace_with_replaces_node_and_updates_id_index() -> Result<()> {
        let html = r#"
        <div id='root'><span id='old'>O</span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const old = document.getElementById('old');
            const neo = document.createElement('span');
            neo.id = 'new';
            neo.textContent = 'N';
            old.replaceWith(neo);
            document.getElementById('result').textContent =
              document.getElementById('root').textContent + ':' +
              document.querySelectorAll('#old').length + ':' +
              document.querySelectorAll('#new').length;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "N:0:1")?;
        Ok(())
    }

    #[test]
    fn insert_adjacent_element_positions_work() -> Result<()> {
        let html = r#"
        <div id='root'><span id='b'>B</span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const b = document.getElementById('b');
            const a = document.createElement('span');
            a.id = 'a';
            a.textContent = 'A';
            const c = document.createElement('span');
            c.id = 'c';
            c.textContent = 'C';
            const d = document.createElement('span');
            d.id = 'd';
            d.textContent = 'D';
            const e = document.createElement('span');
            e.id = 'e';
            e.textContent = 'E';
            b.insertAdjacentElement('beforebegin', a);
            b.insertAdjacentElement('afterbegin', d);
            b.insertAdjacentElement('beforeend', e);
            b.insertAdjacentElement('afterend', c);
            document.getElementById('result').textContent =
              document.getElementById('root').textContent + ':' +
              document.querySelectorAll('#a').length + ':' +
              document.querySelectorAll('#c').length + ':' +
              document.querySelector('#b>#d').textContent + ':' +
              document.querySelector('#b>#e').textContent;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "ADBEC:1:1:D:E")?;
        Ok(())
    }

    #[test]
    fn insert_adjacent_text_positions_and_expression_work() -> Result<()> {
        let html = r#"
        <div id='root'><span id='b'>B</span></div>
        <input id='v' value='Y'>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const b = document.getElementById('b');
            b.insertAdjacentText('beforebegin', 'A');
            b.insertAdjacentText('afterbegin', 'X');
            b.insertAdjacentText('beforeend', document.getElementById('v').value);
            b.insertAdjacentText('afterend', 'C');
            document.getElementById('result').textContent =
              document.getElementById('root').textContent + ':' + b.textContent;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "AXBYC:XBY")?;
        Ok(())
    }

    #[test]
    fn insert_adjacent_html_positions_and_order_work() -> Result<()> {
        let html = r#"
        <div id='root'><span id='b'>B</span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const b = document.getElementById('b');
            b.insertAdjacentHTML('beforebegin', '<i id="y1">Y</i><i id="y2">Z</i>');
            b.insertAdjacentHTML('afterbegin', 'X<span id="x1">X</span>');
            b.insertAdjacentHTML('beforeend', '<span id="x2">W</span><span id="x3">Q</span>');
            b.insertAdjacentHTML('afterend', 'T<em id="t">T</em>');
            document.getElementById('result').textContent =
              document.getElementById('root').textContent + ':' +
              document.querySelectorAll('#y1').length + ':' +
              document.querySelectorAll('#y2').length + ':' +
              document.querySelectorAll('#x1').length + ':' +
              document.querySelectorAll('#x2').length + ':' +
              document.querySelectorAll('#x3').length + ':' +
              document.querySelectorAll('#t').length;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "YZXXBWQTT:1:1:1:1:1:1")?;
        Ok(())
    }

    #[test]
    fn insert_adjacent_html_position_expression_works() -> Result<()> {
        let html = r#"
        <div id='root'><span id='b'>B</span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const b = document.getElementById('b');
            let head = 'beforebegin';
            let inner = 'afterbegin';
            let tail = 'AFTEREND';
            b.insertAdjacentHTML(head, '<i id="head">H</i>');
            b.insertAdjacentHTML(inner, '<i id="mid">M</i>');
            b.insertAdjacentHTML(tail, '<i id="tail">T</i>');
            document.getElementById('result').textContent =
              document.getElementById('root').textContent + ':' +
              document.querySelectorAll('#head').length + ':' +
              document.querySelectorAll('#mid').length + ':' +
              document.querySelectorAll('#tail').length;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "HMBT:1:1:1")?;
        Ok(())
    }

    #[test]
    fn insert_adjacent_html_invalid_position_expression_fails() -> Result<()> {
        let html = r#"
        <div id='root'><span id='b'>B</span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const pos = 'outer';
            const b = document.getElementById('b');
            b.insertAdjacentHTML(pos, '<i>T</i>');
            document.getElementById('result').textContent = 'ok';
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        let err = h.click("#btn").expect_err("invalid position should fail");
        match err {
            Error::ScriptRuntime(msg) => assert!(msg.contains("unsupported insertAdjacentHTML position")),
            other => panic!("unexpected error: {other:?}"),
        }
        Ok(())
    }

    #[test]
    fn inner_html_set_replaces_children_and_updates_id_index() -> Result<()> {
        let html = r#"
        <div id='box'><span id='old'>O</span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            box.innerHTML = '<span id="new">N</span><b>B</b>';
            const same = box.innerHTML === '<span id="new">N</span><b>B</b>';
            document.getElementById('result').textContent =
              box.textContent + ':' +
              document.querySelectorAll('#old').length + ':' +
              document.querySelectorAll('#new').length + ':' +
              same;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "NB:0:1:true")?;
        Ok(())
    }

    #[test]
    fn inner_html_getter_returns_markup_with_text_nodes() -> Result<()> {
        let html = r#"
        <div id='box'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            box.innerHTML = 'A<i id="x">X</i>C';
            document.getElementById('result').textContent =
              box.innerHTML + ':' + document.getElementById('x').textContent;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "A<i id=\"x\">X</i>C:X")?;
        Ok(())
    }

    #[test]
    fn detached_element_id_is_not_queryable_until_attached() -> Result<()> {
        let html = r#"
        <div id='root'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const node = document.createElement('div');
            node.id = 'late';
            document.getElementById('result').textContent =
              document.querySelectorAll('#late').length + ':';
            document.getElementById('root').appendChild(node);
            document.getElementById('result').textContent =
              document.getElementById('result').textContent +
              document.querySelectorAll('#late').length;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "0:1")?;
        Ok(())
    }

    #[test]
    fn create_text_node_append_and_remove_work() -> Result<()> {
        let html = r#"
        <div id='root'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const root = document.getElementById('root');
            const text = document.createTextNode('A');
            root.appendChild(text);
            document.getElementById('result').textContent = root.textContent + ':';
            text.remove();
            document.getElementById('result').textContent =
              document.getElementById('result').textContent + root.textContent;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "A:")?;
        Ok(())
    }

    #[test]
    fn node_remove_detaches_and_updates_id_index() -> Result<()> {
        let html = r#"
        <div id='root'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const root = document.getElementById('root');
            const el = document.createElement('div');
            el.id = 'gone';
            root.appendChild(el);
            el.remove();
            el.remove();
            document.getElementById('result').textContent =
              document.querySelectorAll('#gone').length + ':' + root.textContent;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "0:")?;
        Ok(())
    }

    #[test]
    fn duplicate_id_prefers_first_match_for_id_selector_api() -> Result<()> {
        let html = r#"
        <div id='root'>
          <span id='dup'>first</span>
          <span id='dup'>second</span>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const byId = document.getElementById('dup');
            const all = document.querySelectorAll('#dup').length;
            const bySelector = document.querySelector('#dup');
            document.getElementById('result').textContent =
              byId.textContent + ':' + all + ':' + bySelector.textContent;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "first:2:first")?;
        Ok(())
    }

    #[test]
    fn duplicate_id_returns_next_match_after_removal_of_first() -> Result<()> {
        let html = r#"
        <div id='root'>
          <span id='first'>first</span>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const first = document.getElementById('first');
            first.remove();

            const a = document.createElement('span');
            a.id = 'dup';
            a.textContent = 'a';
            const b = document.createElement('span');
            b.id = 'dup';
            b.textContent = 'b';
            const root = document.getElementById('root');
            root.appendChild(a);
            root.appendChild(b);

            const active = document.getElementById('dup');
            const all = document.querySelectorAll('#dup').length;
            document.getElementById('result').textContent =
              active.textContent + ':' + all + ':' + document.querySelectorAll('#first').length;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "a:2:0")?;
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
    fn selector_group_and_document_order_dedup_work() -> Result<()> {
        let html = r#"
        <div>
          <span id='second'></span>
          <span id='first'></span>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const firstMatch = document.querySelector('#first, #second').id;
            const same = document.querySelectorAll('#first, #first').length;
            const both = document.querySelectorAll('#first, #second').length;
            document.getElementById('result').textContent =
              firstMatch + ':' + same + ':' + both;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "second:1:2")?;
        Ok(())
    }

    #[test]
    fn selector_adjacent_and_general_sibling_combinators_work() -> Result<()> {
        let html = r#"
        <ul id='list'>
          <li id='a' class='item'>A</li>
          <li id='b' class='item'>B</li>
          <li id='c' class='item'>C</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const adjacent = document.querySelector('#a + .item').id;
            const siblings = document.querySelectorAll('#a ~ .item').length;
            document.getElementById('result').textContent = adjacent + ':' + siblings;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "b:2")?;
        Ok(())
    }

    #[test]
    fn selector_compound_tag_id_class_and_attr_work() -> Result<()> {
        let html = r#"
        <div>
          <span id='target' class='x y' data-role='main' data-on='1'></span>
          <span id='other' class='x' data-role='main'></span>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const exact = document.querySelector("span#target.x.y[data-role='main'][data-on]").id;
            const many = document.querySelectorAll("span.x[data-role='main']").length;
            document.getElementById('result').textContent = exact + ':' + many;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "target:2")?;
        Ok(())
    }

    #[test]
    fn selector_attr_operators_work() -> Result<()> {
        let html = r#"
        <div>
          <span id='first'
            data-code='pre-middle-post'
            tags='alpha one beta'
            lang='en-US'></span>
          <span id='second' data-code='other' tags='two three' lang='fr'></span>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const p1 = document.querySelector('[data-code^=\"pre\"]').id;
            const p2 = document.querySelector('[data-code$=\"post\"]').id;
            const p3 = document.querySelector('[data-code*=\"middle\"]').id;
            const p4 = document.querySelector('[tags~=\"one\"]').id;
            const p5 = document.querySelector('[lang|=\"en\"]').id;
            document.getElementById('result').textContent =
              p1 + ':' + p2 + ':' + p3 + ':' + p4 + ':' + p5;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "first:first:first:first:first")?;
        Ok(())
    }

    #[test]
    fn selector_attr_empty_and_case_insensitive_key_work() -> Result<()> {
        let html = r#"
        <div>
          <span id='target' data-empty='' data-flag='X'></span>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const exact = document.querySelector('[data-empty=\"\"]').id;
            const empty = document.querySelector('[data-empty=]').id;
            const keycase = document.querySelector('[DATA-EMPTY=\"\"]').id;
            document.getElementById('result').textContent = exact + ':' + empty + ':' + keycase;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "target:target:target")?;
        Ok(())
    }

    #[test]
    fn selector_universal_selector_matches_first_element() -> Result<()> {
        let html = r#"
        <div id='root'>
          <section id='first'>A</section>
          <p id='second'>B</p>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent = document.querySelector('*').id;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "root")?;
        Ok(())
    }

    #[test]
    fn selector_universal_with_class_selector_work() -> Result<()> {
        let html = r#"
        <main id='root'>
          <p id='first' class='x'>A</p>
          <span id='second' class='x'>B</span>
          <div id='third' class='x'>C</div>
        </main>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent = document.querySelector('*.x').id;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "first")?;
        Ok(())
    }

    #[test]
    fn selector_pseudo_classes_work() -> Result<()> {
        let html = r#"
        <ul id='list'>
          <li id='first' class='item'>A</li>
          <li id='second' class='item'>B</li>
          <li id='third' class='item'>C</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const first = document.querySelector('.item:first-child').id;
            const last = document.querySelector('.item:last-child').id;
            const second = document.querySelector('li:nth-child(2)').id;
            document.getElementById('result').textContent = first + ':' + last + ':' + second;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "first:third:second")?;
        Ok(())
    }

    #[test]
    fn selector_empty_works() -> Result<()> {
        let html = r#"
        <div id='root'><span id='empty'></span><span id='filled'>A</span><span id='nested'><em></em></span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const first = document.querySelector('#root span:empty').id;
            const total = document.querySelectorAll('#root span:empty').length;
            document.getElementById('result').textContent =
              first + ':' + total;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "empty:1")?;
        Ok(())
    }

    #[test]
    fn selector_nth_child_odd_even_work() -> Result<()> {
        let html = r#"
        <ul>
          <li id='one' class='item'>A</li>
          <li id='two' class='item'>B</li>
          <li id='three' class='item'>C</li>
          <li id='four' class='item'>D</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const odd = document.querySelector('li:nth-child(odd)').id;
            const even = document.querySelector('li:nth-child(even)').id;
            document.getElementById('result').textContent = odd + ':' + even;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "one:two")?;
        Ok(())
    }

    #[test]
    fn selector_nth_child_n_work() -> Result<()> {
        let html = r#"
        <ul>
          <li id='one' class='item'>A</li>
          <li id='two' class='item'>B</li>
          <li id='three' class='item'>C</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const every = document.querySelector('li:nth-child(n)').id;
            const count = document.querySelectorAll('li:nth-child( n )').length;
            document.getElementById('result').textContent = every + ':' + count;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "one:3")?;
        Ok(())
    }

    #[test]
    fn selector_parse_rejects_invalid_nth_child() {
        assert!(
            parse_selector_step("li:nth-child(0)").is_err(),
            "nth-child(0) should be invalid in this engine"
        );
        assert!(
            parse_selector_step("li:nth-child(-1)").is_err(),
            "negative nth-child should be invalid in this engine"
        );
        assert!(
            parse_selector_step("li:nth-child(2n+)").is_err(),
            "malformed expression nth-child should be rejected"
        );
        assert!(
            parse_selector_step("li:nth-child(n1)").is_err(),
            "invalid expression nth-child should be rejected"
        );
        assert!(
            parse_selector_step("li:nth-last-child(n1)").is_err(),
            "invalid expression nth-last-child should be rejected"
        );
        assert!(
            parse_selector_step("li:nth-last-child(0)").is_err(),
            "nth-last-child(0) should be invalid in this engine"
        );
        assert!(
            parse_selector_step("li:nth-last-child(2n+)").is_err(),
            "malformed expression nth-last-child should be rejected"
        );
        assert!(
            parse_selector_step("li:nth-of-type(0)").is_err(),
            "nth-of-type(0) should be invalid in this engine"
        );
        assert!(
            parse_selector_step("li:nth-of-type(2n+)").is_err(),
            "malformed expression nth-of-type should be rejected"
        );
        assert!(
            parse_selector_step("li:nth-last-of-type(2n+)").is_err(),
            "malformed expression nth-last-of-type should be rejected"
        );
        assert!(
            parse_selector_step("li:nth-last-of-type(0)").is_err(),
            "nth-last-of-type(0) should be invalid in this engine"
        );
        assert!(
            parse_selector_step("li:not()").is_err(),
            "empty :not should be invalid"
        );

        assert_eq!(
            split_selector_groups("li:not([data='a,b']) , #x").map(|groups| groups.len()),
            Ok(2)
        );
        assert_eq!(
            parse_selector_groups("li:not(.skip, #first), #x").map(|groups| groups.len()),
            Ok(2)
        );
    }

    #[test]
    fn selector_nth_child_an_plus_b_work() -> Result<()> {
        let html = r#"
        <ul>
          <li id='one' class='item'>A</li>
          <li id='two' class='item'>B</li>
          <li id='three' class='item'>C</li>
          <li id='four' class='item'>D</li>
          <li id='five' class='item'>E</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const first_odd = document.querySelector('li:nth-child(2n+1)').id;
            const odd_count = document.querySelectorAll('li:nth-child(2n+1)').length;
            const shifted = document.querySelector('li:nth-child(-n+3)').id;
            document.getElementById('result').textContent = first_odd + ':' + odd_count + ':' + shifted;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "one:3:one")?;
        Ok(())
    }

    #[test]
    fn selector_nth_last_child_an_plus_b_work() -> Result<()> {
        let html = r#"
        <ul>
          <li id='one' class='item'>A</li>
          <li id='two' class='item'>B</li>
          <li id='three' class='item'>C</li>
          <li id='four' class='item'>D</li>
          <li id='five' class='item'>E</li>
          <li id='six' class='item'>F</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const first = document.querySelector('li:nth-last-child(2n+1)').id;
            const count = document.querySelectorAll('li:nth-last-child(2n+1)').length;
            const shifted = document.querySelector('li:nth-last-child(-n+3)').id;
            document.getElementById('result').textContent = first + ':' + count + ':' + shifted;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "two:3:four")?;
        Ok(())
    }

    #[test]
    fn selector_nth_of_type_works() -> Result<()> {
        let html = r#"
        <ul id='list'>
          <li id='first-li'>A</li>
          <span id='only-span'>S</span>
          <li id='second-li'>B</li>
          <em id='not-li'>E</em>
          <li id='third-li'>C</li>
          <li id='fourth-li'>D</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const odd = document.querySelector('li:nth-of-type(odd)').id;
            const even = document.querySelector('li:nth-of-type(even)').id;
            const exact = document.querySelector('li:nth-of-type(3)').id;
            const expression = document.querySelectorAll('li:nth-of-type(2n)').length;
            document.getElementById('result').textContent = odd + ':' + even + ':' + exact + ':' + expression;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "first-li:second-li:third-li:2")?;
        Ok(())
    }

    #[test]
    fn selector_nth_of_type_n_works() -> Result<()> {
        let html = r#"
        <ul id='list'>
          <li id='first-li'>A</li>
          <span id='only-span'>S</span>
          <li id='second-li'>B</li>
          <em id='not-li'>E</em>
          <li id='third-li'>C</li>
          <li id='fourth-li'>D</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const first = document.querySelector('li:nth-of-type(n)').id;
            const all = document.querySelectorAll('li:nth-of-type(n)').length;
            document.getElementById('result').textContent = first + ':' + all;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "first-li:4")?;
        Ok(())
    }

    #[test]
    fn selector_nth_last_of_type_works() -> Result<()> {
        let html = r#"
        <ul id='list'>
          <li id='first-li'>A</li>
          <span id='only-span'>S</span>
          <li id='second-li'>B</li>
          <em id='not-li'>E</em>
          <li id='third-li'>C</li>
          <li id='fourth-li'>D</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const odd = document.querySelector('li:nth-last-of-type(odd)').id;
            const even = document.querySelector('li:nth-last-of-type(even)').id;
            const exact = document.querySelector('li:nth-last-of-type(2)').id;
            const expression = document.querySelectorAll('li:nth-last-of-type(2n)').length;
            document.getElementById('result').textContent = odd + ':' + even + ':' + exact + ':' + expression;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "second-li:first-li:third-li:2")?;
        Ok(())
    }

    #[test]
    fn selector_nth_last_of_type_n_works() -> Result<()> {
        let html = r#"
        <ul id='list'>
          <li id='first-li'>A</li>
          <span id='only-span'>S</span>
          <li id='second-li'>B</li>
          <em id='not-li'>E</em>
          <li id='third-li'>C</li>
          <li id='fourth-li'>D</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const first = document.querySelector('li:nth-last-of-type(n)').id;
            const all = document.querySelectorAll('li:nth-last-of-type(n)').length;
            document.getElementById('result').textContent = first + ':' + all;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "first-li:4")?;
        Ok(())
    }

    #[test]
    fn selector_not_works() -> Result<()> {
        let html = r#"
        <div id='root'>
          <span id='a' class='target'>A</span>
          <span id='b'>B</span>
          <span id='c' class='target'>C</span>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const first = document.querySelector('span:not(.target)').id;
            const total = document.querySelectorAll('span:not(.target)').length;
            const explicit = document.querySelector('span:not(#b)').id;
            document.getElementById('result').textContent = first + ':' + total + ':' + explicit;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "b:1:a")?;
        Ok(())
    }

    #[test]
    fn selector_nested_not_works() -> Result<()> {
        let html = r#"
        <div id='root'>
          <li id='a' class='target'>A</li>
          <li id='b'>B</li>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const matched = document.querySelector('li:not(:not(.target))').id;
            const total = document.querySelectorAll('li:not(:not(.target))').length;
            document.getElementById('result').textContent = matched + ':' + total;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "a:1")?;
        Ok(())
    }

    #[test]
    fn selector_not_with_multiple_selectors_works() -> Result<()> {
        let html = r#"
        <div id='root'>
          <li id='first' class='target'>A</li>
          <li id='middle'>B</li>
          <li id='skip' class='skip'>C</li>
          <li id='last'>D</li>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const first = document.querySelector('li:not(.skip, #first)').id;
            const total = document.querySelectorAll('li:not(.skip, #first)').length;
            document.getElementById('result').textContent = first + ':' + total;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "middle:2")?;
        Ok(())
    }

    #[test]
    fn selector_not_with_complex_selector_list_works() -> Result<()> {
        let html = r#"
        <div id='root'>
          <div id='forbidden' class='scope'>
            <span id='forbidden-a'>A</span>
            <span id='forbidden-b'>B</span>
          </div>
          <span id='skip-me'>C</span>
          <div id='safe'>
            <span id='safe-a'>D</span>
          </div>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const first = document.querySelector('span:not(.scope *, #skip-me)').id;
            const total = document.querySelectorAll('span:not(.scope *, #skip-me)').length;
            document.getElementById('result').textContent = first + ':' + total;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "safe-a:1")?;
        Ok(())
    }

    #[test]
    fn selector_not_with_complex_selector_adjacent_combinator_works() -> Result<()> {
        let html = r#"
        <div id='root'>
          <div id='scope' class='scope'></div>
          <span id='excluded'>A</span>
          <span id='included'>B</span>
          <span id='included-2'>C</span>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const first = document.querySelector('span:not(.scope + span)').id;
            const total = document.querySelectorAll('span:not(.scope + span)').length;
            document.getElementById('result').textContent = first + ':' + total;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "included:2")?;
        Ok(())
    }

    #[test]
    fn selector_not_with_complex_selector_general_sibling_combinator_works() -> Result<()> {
        let html = r#"
        <div id='root'>
          <span id='included-before'>A</span>
          <div id='scope' class='scope'></div>
          <span id='excluded-1'>B</span>
          <span id='excluded-2'>C</span>
          <p>between</p>
          <span id='excluded-3'>D</span>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const first = document.querySelector('span:not(.scope ~ span)').id;
            const total = document.querySelectorAll('span:not(.scope ~ span)').length;
            document.getElementById('result').textContent = first + ':' + total;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "included-before:1")?;
        Ok(())
    }

    #[test]
    fn selector_not_with_complex_selector_list_general_sibling_works() -> Result<()> {
        let html = r#"
        <div id='root'>
          <span id='included-before'>A</span>
          <div id='scope' class='scope'></div>
          <span id='excluded-id'>B</span>
          <span id='excluded-sibling'>C</span>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const first = document.querySelector('span:not(.scope ~ span, #excluded-id)').id;
            const total = document.querySelectorAll('span:not(.scope ~ span, #excluded-id)').length;
            document.getElementById('result').textContent = first + ':' + total;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "included-before:1")?;
        Ok(())
    }

    #[test]
    fn selector_not_with_complex_selector_child_combinator_works() -> Result<()> {
        let html = r#"
        <div id='root'>
          <div id='scope' class='scope'>
            <span id='excluded'>A</span>
          </div>
          <span id='included'>B</span>
          <span id='included-2'>C</span>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const first = document.querySelector('span:not(.scope > span)').id;
            const total = document.querySelectorAll('span:not(.scope > span)').length;
            document.getElementById('result').textContent = first + ':' + total;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "included:2")?;
        Ok(())
    }

    #[test]
    fn selector_not_with_multiple_not_chain_works() -> Result<()> {
        let html = r#"
        <div id='root'>
          <li id='both' class='foo bar'>A</li>
          <li id='foo-only' class='foo'>B</li>
          <li id='bar-only' class='bar'>C</li>
          <li id='none'>D</li>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const first = document.querySelector('li:not(:not(.foo), :not(.bar))').id;
            const total = document.querySelectorAll('li:not(:not(.foo), :not(.bar))').length;
            document.getElementById('result').textContent = first + ':' + total;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "both:1")?;
        Ok(())
    }

    #[test]
    fn selector_first_last_of_type_works() -> Result<()> {
        let html = r#"
        <div id='root'>
          <p id='first-p'>A</p>
          <span id='first-span'>B</span>
          <p id='last-p'>C</p>
          <span id='middle-span'>D</span>
          <span id='last-span'>E</span>
          <li id='first-li'>F</li>
          <li id='last-li'>G</li>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const firstSpan = document.querySelector('span:first-of-type').id;
            const lastSpan = document.querySelector('span:last-of-type').id;
            const firstP = document.querySelector('p:first-of-type').id;
            const lastP = document.querySelector('p:last-of-type').id;
            const firstLi = document.querySelector('li:first-of-type').id;
            const lastLi = document.querySelector('li:last-of-type').id;
            document.getElementById('result').textContent = firstSpan + ':' + lastSpan + ':' + firstP + ':' + lastP + ':' + firstLi + ':' + lastLi;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text(
            "#result",
            "first-span:last-span:first-p:last-p:first-li:last-li",
        )?;
        Ok(())
    }

    #[test]
    fn selector_only_child_and_only_of_type_works() -> Result<()> {
        let html = r#"
        <div id='root'>
          <div id='single-p'>
            <p id='lonely-p'>A</p>
          </div>
          <div id='group'>
            <span id='only-span'>B</span>
          </div>
          <section id='mixed-of-type'>
            <span id='mixed-only-span'>C</span>
            <em id='mixed-only-em'>D</em>
          </section>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const lonely = document.querySelector('p:only-child').id;
            const onlySpanInGroup = document.querySelector('#group span:only-child').id;
            const onlySpanOfType = document.querySelector('#mixed-of-type span:only-of-type').id;
            const onlyEmOfType = document.querySelector('#mixed-of-type em:only-of-type').id;
            document.getElementById('result').textContent = lonely + ':' + onlySpanInGroup + ':' + onlySpanOfType + ':' + onlyEmOfType;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text(
            "#result",
            "lonely-p:only-span:mixed-only-span:mixed-only-em",
        )?;
        Ok(())
    }

    #[test]
    fn selector_checked_disabled_enabled_works() -> Result<()> {
        let html = r#"
        <input id='enabled' value='ok'>
        <input id='disabled' disabled value='ng'>
        <input id='unchecked' type='checkbox'>
        <input id='checked' type='checkbox' checked>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const checked = document.querySelector('input:checked').id;
            const disabled = document.querySelector('input:disabled').id;
            const enabled = document.querySelector('input:enabled').id;
            document.getElementById('result').textContent = checked + ':' + disabled + ':' + enabled;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "checked:disabled:enabled")?;
        Ok(())
    }

    #[test]
    fn selector_required_optional_readonly_readwrite_works() -> Result<()> {
        let html = r#"
        <input id='r' required value='r'>
        <input id='o'>
        <input id='ro' readonly>
        <input id='rw'>
        <input id='r2'>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const required = document.querySelector('input:required').id;
            const optional = document.querySelector('input:optional').id;
            const readOnly = document.querySelector('input:readonly').id;
            const readWrite = document.querySelector('input:read-write').id;
            const summary =
              required + ':' + optional + ':' + readOnly + ':' + readWrite;
            document.getElementById('r').required = false;
            document.getElementById('r2').required = true;
            const afterRequired = document.querySelector('input:required').id;
            const afterOptional = document.querySelector('input:optional').id;
            document.getElementById('result').textContent =
              summary + ':' + afterRequired + ':' + afterOptional;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "r:o:ro:r:r2:r")?;
        Ok(())
    }

    #[test]
    fn selector_trailing_group_separator_is_rejected() -> Result<()> {
        let html = r#"<div id='x'></div>"#;
        let h = Harness::from_html(html)?;
        let err = h
            .assert_exists("#x,")
            .expect_err("selector should be invalid");
        match err {
            Error::UnsupportedSelector(selector) => assert_eq!(selector, "#x,"),
            other => panic!("unexpected error: {other:?}"),
        }
        Ok(())
    }

    #[test]
    fn selector_parse_supports_nth_child_single_arg() {
        let step = parse_selector_step("li:nth-child(2)").expect("parse should succeed");
        assert_eq!(step.tag, Some("li".into()));
        assert_eq!(
            step.pseudo_classes,
            vec![SelectorPseudoClass::NthChild(NthChildSelector::Exact(2))]
        );
    }

    #[test]
    fn selector_parse_supports_nth_child_odd_even() {
        let odd = parse_selector_step("li:nth-child(odd)").expect("parse should succeed");
        let even = parse_selector_step("li:nth-child(even)").expect("parse should succeed");
        assert_eq!(
            odd.pseudo_classes,
            vec![SelectorPseudoClass::NthChild(NthChildSelector::Odd)]
        );
        assert_eq!(
            even.pseudo_classes,
            vec![SelectorPseudoClass::NthChild(NthChildSelector::Even)]
        );
    }

    #[test]
    fn selector_parse_supports_nth_child_n() {
        let n = parse_selector_step("li:nth-child(n)").expect("parse should succeed");
        assert_eq!(
            n.pseudo_classes,
            vec![SelectorPseudoClass::NthChild(NthChildSelector::AnPlusB(
                1, 0
            ))]
        );
    }

    #[test]
    fn selector_parse_supports_nth_last_child_an_plus_b() {
        let direct = parse_selector_step("li:nth-last-child(2n+1)").expect("parse should succeed");
        assert_eq!(
            direct.pseudo_classes,
            vec![SelectorPseudoClass::NthLastChild(
                NthChildSelector::AnPlusB(2, 1)
            )]
        );
    }

    #[test]
    fn selector_parse_supports_nth_last_child_odd_even_exact() {
        let odd = parse_selector_step("li:nth-last-child(odd)").expect("parse should succeed");
        let even = parse_selector_step("li:nth-last-child(even)").expect("parse should succeed");
        let exact = parse_selector_step("li:nth-last-child(2)").expect("parse should succeed");
        assert_eq!(
            odd.pseudo_classes,
            vec![SelectorPseudoClass::NthLastChild(NthChildSelector::Odd)]
        );
        assert_eq!(
            even.pseudo_classes,
            vec![SelectorPseudoClass::NthLastChild(NthChildSelector::Even)]
        );
        assert_eq!(
            exact.pseudo_classes,
            vec![SelectorPseudoClass::NthLastChild(NthChildSelector::Exact(
                2
            ))]
        );
    }

    #[test]
    fn selector_parse_supports_nth_child_an_plus_b() {
        let direct = parse_selector_step("li:nth-child(2n+1)").expect("parse should succeed");
        let shifted = parse_selector_step("li:nth-child(-n+3)").expect("parse should succeed");
        assert_eq!(
            direct.pseudo_classes,
            vec![SelectorPseudoClass::NthChild(NthChildSelector::AnPlusB(
                2, 1
            ))]
        );
        assert_eq!(
            shifted.pseudo_classes,
            vec![SelectorPseudoClass::NthChild(NthChildSelector::AnPlusB(
                -1, 3
            ))]
        );
    }

    #[test]
    fn selector_parse_supports_first_last_of_type() {
        let first = parse_selector_step("li:first-of-type").expect("parse should succeed");
        let last = parse_selector_step("li:last-of-type").expect("parse should succeed");
        assert_eq!(first.pseudo_classes, vec![SelectorPseudoClass::FirstOfType]);
        assert_eq!(last.pseudo_classes, vec![SelectorPseudoClass::LastOfType]);
    }

    #[test]
    fn selector_parse_supports_empty() {
        let parsed = parse_selector_step("span:empty").expect("parse should succeed");
        assert_eq!(parsed.pseudo_classes, vec![SelectorPseudoClass::Empty]);
    }

    #[test]
    fn selector_parse_supports_only_child_and_only_of_type() {
        let only_child = parse_selector_step("li:only-child").expect("parse should succeed");
        let only_of_type = parse_selector_step("li:only-of-type").expect("parse should succeed");
        assert_eq!(
            only_child.pseudo_classes,
            vec![SelectorPseudoClass::OnlyChild]
        );
        assert_eq!(
            only_of_type.pseudo_classes,
            vec![SelectorPseudoClass::OnlyOfType]
        );
    }

    #[test]
    fn selector_parse_supports_checked_disabled_enabled() {
        let checked = parse_selector_step("input:checked").expect("parse should succeed");
        let disabled = parse_selector_step("input:disabled").expect("parse should succeed");
        let enabled = parse_selector_step("input:enabled").expect("parse should succeed");
        assert_eq!(checked.pseudo_classes, vec![SelectorPseudoClass::Checked]);
        assert_eq!(disabled.pseudo_classes, vec![SelectorPseudoClass::Disabled]);
        assert_eq!(enabled.pseudo_classes, vec![SelectorPseudoClass::Enabled]);
    }

    #[test]
    fn selector_parse_supports_required_optional_readonly_readwrite() {
        let required = parse_selector_step("input:required").expect("parse should succeed");
        let optional = parse_selector_step("input:optional").expect("parse should succeed");
        let read_only = parse_selector_step("input:read-only").expect("parse should succeed");
        let read_only_alias = parse_selector_step("input:readonly").expect("parse should succeed");
        let read_write = parse_selector_step("input:read-write").expect("parse should succeed");
        assert_eq!(required.pseudo_classes, vec![SelectorPseudoClass::Required]);
        assert_eq!(optional.pseudo_classes, vec![SelectorPseudoClass::Optional]);
        assert_eq!(
            read_only.pseudo_classes,
            vec![SelectorPseudoClass::Readonly]
        );
        assert_eq!(
            read_only_alias.pseudo_classes,
            vec![SelectorPseudoClass::Readonly]
        );
        assert_eq!(
            read_write.pseudo_classes,
            vec![SelectorPseudoClass::Readwrite]
        );
    }

    #[test]
    fn selector_parse_supports_focus_and_focus_within() {
        let focus = parse_selector_step("input:focus").expect("parse should succeed");
        let focus_within = parse_selector_step("div:focus-within").expect("parse should succeed");
        assert_eq!(focus.pseudo_classes, vec![SelectorPseudoClass::Focus]);
        assert_eq!(
            focus_within.pseudo_classes,
            vec![SelectorPseudoClass::FocusWithin]
        );
    }

    #[test]
    fn selector_parse_supports_active() {
        let active = parse_selector_step("button:active").expect("parse should succeed");
        assert_eq!(active.pseudo_classes, vec![SelectorPseudoClass::Active]);
    }

    #[test]
    fn selector_parse_supports_not() {
        let by_id = parse_selector_step("span:not(#x)").expect("parse should succeed");
        let by_class = parse_selector_step("span:not(.x)").expect("parse should succeed");
        let nested = parse_selector_step("span:not(:not(.x))").expect("parse should succeed");
        let with_attribute =
            parse_selector_step("li:not([data='a,b'])").expect("parse should succeed");
        if let SelectorPseudoClass::Not(inners) = &by_id.pseudo_classes[0] {
            assert_eq!(inners.len(), 1);
            assert_eq!(inners[0].len(), 1);
            assert_eq!(inners[0][0].step.id.as_deref(), Some("x"));
        } else {
            panic!("expected not pseudo");
        }
        if let SelectorPseudoClass::Not(inners) = &by_class.pseudo_classes[0] {
            assert_eq!(inners.len(), 1);
            assert_eq!(inners[0].len(), 1);
            assert_eq!(inners[0][0].step.tag.as_deref(), None);
            assert_eq!(inners[0][0].step.classes.as_slice(), &["x"]);
        } else {
            panic!("expected not pseudo");
        }
        if let SelectorPseudoClass::Not(inners) = &nested.pseudo_classes[0] {
            assert_eq!(inners.len(), 1);
            assert_eq!(inners[0].len(), 1);
            if let SelectorPseudoClass::Not(inner_inners) = &inners[0][0].step.pseudo_classes[0] {
                assert_eq!(inner_inners.len(), 1);
                assert_eq!(inner_inners[0][0].step.tag.as_deref(), None);
                assert_eq!(inner_inners[0][0].step.classes.as_slice(), &["x"]);
                assert!(inner_inners[0][0].step.pseudo_classes.is_empty());
            } else {
                panic!("expected nested not pseudo");
            }
        } else {
            panic!("expected not pseudo");
        }
        if let SelectorPseudoClass::Not(inners) = &with_attribute.pseudo_classes[0] {
            assert_eq!(inners.len(), 1);
            assert_eq!(inners[0].len(), 1);
            let inner = &inners[0][0].step;
            assert_eq!(
                inner.attrs,
                vec![SelectorAttrCondition::Eq {
                    key: "data".into(),
                    value: "a,b".into()
                }]
            );
            assert!(inner.classes.is_empty());
            assert!(inner.id.is_none());
            assert!(inner.pseudo_classes.is_empty());
            assert!(!inner.universal);
        } else {
            panic!("expected not pseudo");
        }
    }

    #[test]
    fn selector_parse_supports_where_is_and_has() {
        let where_step = parse_selector_step("span:where(.a, #b, :not(.skip))")
            .expect("parse should succeed");
        let is_step = parse_selector_step("span:is(.a, #b, :not(.skip))")
            .expect("parse should succeed");
        let has_step = parse_selector_step("section:has(.c, #d)")
            .expect("parse should succeed");

        assert!(matches!(where_step.pseudo_classes[0], SelectorPseudoClass::Where(_)));
        if let SelectorPseudoClass::Where(inners) = &where_step.pseudo_classes[0] {
            assert_eq!(inners.len(), 3);
            assert_eq!(inners[0].len(), 1);
            assert_eq!(inners[1].len(), 1);
            assert_eq!(inners[2].len(), 1);
        }

        assert!(matches!(is_step.pseudo_classes[0], SelectorPseudoClass::Is(_)));
        assert!(matches!(has_step.pseudo_classes[0], SelectorPseudoClass::Has(_)));
    }

    #[test]
    fn selector_parse_supports_attribute_operators() {
        let exists = parse_selector_step("[flag]").expect("parse should succeed");
        let eq = parse_selector_step("[data='value']").expect("parse should succeed");
        let starts_with = parse_selector_step("[data^='pre']").expect("parse should succeed");
        let ends_with = parse_selector_step("[data$='post']").expect("parse should succeed");
        let contains = parse_selector_step("[data*='med']").expect("parse should succeed");
        let includes = parse_selector_step("[tags~='one']").expect("parse should succeed");
        let dash = parse_selector_step("[lang|='en']").expect("parse should succeed");

        assert_eq!(
            exists.attrs,
            vec![SelectorAttrCondition::Exists { key: "flag".into() }]
        );
        assert_eq!(
            eq.attrs,
            vec![SelectorAttrCondition::Eq {
                key: "data".into(),
                value: "value".into()
            }]
        );
        assert_eq!(
            starts_with.attrs,
            vec![SelectorAttrCondition::StartsWith {
                key: "data".into(),
                value: "pre".into()
            }]
        );
        assert_eq!(
            ends_with.attrs,
            vec![SelectorAttrCondition::EndsWith {
                key: "data".into(),
                value: "post".into()
            }]
        );
        assert_eq!(
            contains.attrs,
            vec![SelectorAttrCondition::Contains {
                key: "data".into(),
                value: "med".into()
            }]
        );
        assert_eq!(
            includes.attrs,
            vec![SelectorAttrCondition::Includes {
                key: "tags".into(),
                value: "one".into()
            }]
        );
        assert_eq!(
            dash.attrs,
            vec![SelectorAttrCondition::DashMatch {
                key: "lang".into(),
                value: "en".into()
            }]
        );
        let empty = parse_selector_step("[data='']").expect("parse should succeed");
        let case_key = parse_selector_step("[DATA='v']").expect("parse should succeed");
        let unquoted_empty = parse_selector_step("[data=]").expect("parse should succeed");
        assert_eq!(
            empty.attrs,
            vec![SelectorAttrCondition::Eq {
                key: "data".into(),
                value: "".into()
            }]
        );
        assert_eq!(
            case_key.attrs,
            vec![SelectorAttrCondition::Eq {
                key: "data".into(),
                value: "v".into()
            }]
        );
        assert_eq!(
            unquoted_empty.attrs,
            vec![SelectorAttrCondition::Eq {
                key: "data".into(),
                value: "".into()
            }]
        );
    }

    #[test]
    fn selector_parse_supports_not_with_multiple_selectors() {
        let multi =
            parse_selector_step("li:not(.a, #target, :not(.skip))").expect("parse should succeed");
        let SelectorPseudoClass::Not(inners) = &multi.pseudo_classes[0] else {
            panic!("expected not pseudo");
        };
        assert_eq!(inners.len(), 3);
        assert_eq!(inners[0].len(), 1);
        assert_eq!(inners[0][0].step.classes.as_slice(), &["a"]);

        assert_eq!(inners[1].len(), 1);
        assert_eq!(inners[1][0].step.id.as_deref(), Some("target"));

        assert_eq!(inners[2].len(), 1);
        assert_eq!(inners[2][0].step.pseudo_classes.len(), 1);
        let inner = &inners[2][0].step.pseudo_classes[0];
        assert!(matches!(inner, SelectorPseudoClass::Not(_)));
    }

    #[test]
    fn selector_parse_supports_not_with_multiple_not_pseudos() {
        let parsed =
            parse_selector_step("li:not(:not(.foo), :not(.bar))").expect("parse should succeed");
        let SelectorPseudoClass::Not(inners) = &parsed.pseudo_classes[0] else {
            panic!("expected not pseudo");
        };

        assert_eq!(inners.len(), 2);

        assert_eq!(inners[0].len(), 1);
        assert_eq!(inners[0][0].step.pseudo_classes.len(), 1);
        let first = &inners[0][0].step.pseudo_classes[0];
        if let SelectorPseudoClass::Not(inner_inners) = first {
            assert_eq!(inner_inners.len(), 1);
            assert_eq!(inner_inners[0][0].step.classes.as_slice(), &["foo"]);
        } else {
            panic!("expected nested not pseudo in first arg");
        }

        assert_eq!(inners[1].len(), 1);
        assert_eq!(inners[1][0].step.pseudo_classes.len(), 1);
        let second = &inners[1][0].step.pseudo_classes[0];
        if let SelectorPseudoClass::Not(inner_inners) = second {
            assert_eq!(inner_inners.len(), 1);
            assert_eq!(inner_inners[0][0].step.classes.as_slice(), &["bar"]);
        } else {
            panic!("expected nested not pseudo in second arg");
        }
    }

    #[test]
    fn selector_parse_supports_not_with_complex_selector_list() {
        let parsed = parse_selector_step("span:not(.scope *, #skip-me, .area :not(.nested .leaf))")
            .expect("parse should succeed");
        let SelectorPseudoClass::Not(inners) = &parsed.pseudo_classes[0] else {
            panic!("expected not pseudo");
        };

        assert_eq!(inners.len(), 3);

        assert_eq!(inners[0].len(), 2);
        assert_eq!(inners[0][0].step.classes.as_slice(), &["scope"]);
        assert!(inners[0][0].combinator.is_none());
        assert_eq!(inners[0][1].step.tag.as_deref(), None);
        assert!(inners[0][1].step.universal);
        assert_eq!(
            inners[0][1].combinator,
            Some(SelectorCombinator::Descendant)
        );

        assert_eq!(inners[1].len(), 1);
        assert_eq!(inners[1][0].step.id.as_deref(), Some("skip-me"));
        assert!(inners[1][0].combinator.is_none());

        assert_eq!(inners[2].len(), 2);
        assert_eq!(inners[2][0].step.classes.as_slice(), &["area"]);
        assert_eq!(inners[2][1].step.pseudo_classes.len(), 1);
        let nested = &inners[2][1].step.pseudo_classes[0];
        if let SelectorPseudoClass::Not(nested_inners) = nested {
            assert_eq!(nested_inners.len(), 1);
            assert_eq!(nested_inners[0].len(), 2);
            assert_eq!(nested_inners[0][0].step.classes.as_slice(), &["nested"]);
            assert_eq!(nested_inners[0][1].step.classes.as_slice(), &["leaf"]);
            assert_eq!(
                nested_inners[0][1].combinator,
                Some(SelectorCombinator::Descendant)
            );
        } else {
            panic!("expected nested not pseudo");
        }
    }

    #[test]
    fn selector_parse_supports_not_with_adjacent_selector() {
        let parsed = parse_selector_step("span:not(.scope + span)").expect("parse should succeed");
        let SelectorPseudoClass::Not(inners) = &parsed.pseudo_classes[0] else {
            panic!("expected not pseudo");
        };

        assert_eq!(inners.len(), 1);
        assert_eq!(inners[0].len(), 2);
        assert_eq!(inners[0][0].step.classes.as_slice(), &["scope"]);
        assert_eq!(inners[0][1].step.tag.as_deref(), Some("span"));
        assert_eq!(
            inners[0][1].combinator,
            Some(SelectorCombinator::AdjacentSibling)
        );
    }

    #[test]
    fn selector_parse_supports_not_with_selector_list_general_sibling_selector() {
        let parsed = parse_selector_step("span:not(.scope ~ span, #excluded-id)")
            .expect("parse should succeed");
        let SelectorPseudoClass::Not(inners) = &parsed.pseudo_classes[0] else {
            panic!("expected not pseudo");
        };

        assert_eq!(inners.len(), 2);
        assert_eq!(inners[0].len(), 2);
        assert_eq!(inners[0][0].step.classes.as_slice(), &["scope"]);
        assert_eq!(inners[0][1].step.tag.as_deref(), Some("span"));
        assert_eq!(
            inners[0][1].combinator,
            Some(SelectorCombinator::GeneralSibling)
        );

        assert_eq!(inners[1].len(), 1);
        assert_eq!(inners[1][0].step.id.as_deref(), Some("excluded-id"));
        assert!(inners[1][0].combinator.is_none());
    }

    #[test]
    fn selector_parse_supports_not_with_general_sibling_selector() {
        let parsed = parse_selector_step("span:not(.scope ~ span)").expect("parse should succeed");
        let SelectorPseudoClass::Not(inners) = &parsed.pseudo_classes[0] else {
            panic!("expected not pseudo");
        };

        assert_eq!(inners.len(), 1);
        assert_eq!(inners[0].len(), 2);
        assert_eq!(inners[0][0].step.classes.as_slice(), &["scope"]);
        assert_eq!(inners[0][1].step.tag.as_deref(), Some("span"));
        assert_eq!(
            inners[0][1].combinator,
            Some(SelectorCombinator::GeneralSibling)
        );
    }

    #[test]
    fn selector_parse_supports_not_with_child_selector() {
        let parsed = parse_selector_step("span:not(.scope > span)").expect("parse should succeed");
        let SelectorPseudoClass::Not(inners) = &parsed.pseudo_classes[0] else {
            panic!("expected not pseudo");
        };

        assert_eq!(inners.len(), 1);
        assert_eq!(inners[0].len(), 2);
        assert_eq!(inners[0][0].step.classes.as_slice(), &["scope"]);
        assert_eq!(inners[0][1].step.tag.as_deref(), Some("span"));
        assert_eq!(inners[0][1].combinator, Some(SelectorCombinator::Child));
    }

    #[test]
    fn selector_parse_rejects_invalid_not_argument_forms() {
        assert!(parse_selector_step("span:not()").is_err());
        assert!(parse_selector_step("span:not(,)").is_err());
        assert!(parse_selector_step("span:not(.a,,#b)").is_err());
        assert!(parse_selector_step("span:not(.a,").is_err());
        assert!(parse_selector_step("span:not(.a,#b,)").is_err());
    }

    #[test]
    fn selector_parse_rejects_unclosed_not_parenthesis() {
        assert!(parse_selector_step("span:not(.a, #b").is_err());
        assert!(parse_selector_step("span:not(:not(.a)").is_err());
    }

    #[test]
    fn selector_runtime_rejects_invalid_not_selector() -> Result<()> {
        let html = "<div id='root'></div>";
        let h = Harness::from_html(html)?;

        let err = h
            .assert_exists("span:not()")
            .expect_err("invalid selector should be rejected");
        match err {
            Error::UnsupportedSelector(selector) => assert_eq!(selector, "span:not()"),
            other => panic!("expected unsupported selector error, got: {other:?}"),
        }

        let err = h
            .assert_exists("span:not(.a,)")
            .expect_err("invalid selector should be rejected");
        match err {
            Error::UnsupportedSelector(selector) => assert_eq!(selector, "span:not(.a,)"),
            other => panic!("expected unsupported selector error, got: {other:?}"),
        }

        Ok(())
    }

    #[test]
    fn selector_parse_supports_nth_of_type() {
        let odd = parse_selector_step("li:nth-of-type(odd)").expect("parse should succeed");
        let expr = parse_selector_step("li:nth-of-type(2n)").expect("parse should succeed");
        let n = parse_selector_step("li:nth-of-type(n)").expect("parse should succeed");
        let exact = parse_selector_step("li:nth-of-type(3)").expect("parse should succeed");
        assert_eq!(
            odd.pseudo_classes,
            vec![SelectorPseudoClass::NthOfType(NthChildSelector::Odd)]
        );
        assert_eq!(
            expr.pseudo_classes,
            vec![SelectorPseudoClass::NthOfType(NthChildSelector::AnPlusB(
                2, 0
            ))]
        );
        assert_eq!(
            n.pseudo_classes,
            vec![SelectorPseudoClass::NthOfType(NthChildSelector::AnPlusB(
                1, 0
            ))]
        );
        assert_eq!(
            exact.pseudo_classes,
            vec![SelectorPseudoClass::NthOfType(NthChildSelector::Exact(3))]
        );
    }

    #[test]
    fn selector_parse_supports_nth_last_of_type() {
        let odd = parse_selector_step("li:nth-last-of-type(odd)").expect("parse should succeed");
        let even = parse_selector_step("li:nth-last-of-type(even)").expect("parse should succeed");
        let n = parse_selector_step("li:nth-last-of-type(n)").expect("parse should succeed");
        let exact = parse_selector_step("li:nth-last-of-type(2)").expect("parse should succeed");
        assert_eq!(
            odd.pseudo_classes,
            vec![SelectorPseudoClass::NthLastOfType(NthChildSelector::Odd)]
        );
        assert_eq!(
            even.pseudo_classes,
            vec![SelectorPseudoClass::NthLastOfType(NthChildSelector::Even)]
        );
        assert_eq!(
            n.pseudo_classes,
            vec![SelectorPseudoClass::NthLastOfType(
                NthChildSelector::AnPlusB(1, 0)
            )]
        );
        assert_eq!(
            exact.pseudo_classes,
            vec![SelectorPseudoClass::NthLastOfType(NthChildSelector::Exact(
                2
            ))]
        );
    }

    #[test]
    fn selector_nth_last_child_odd_even_work() -> Result<()> {
        let html = r#"
        <ul>
          <li id='one' class='item'>A</li>
          <li id='two' class='item'>B</li>
          <li id='three' class='item'>C</li>
          <li id='four' class='item'>D</li>
          <li id='five' class='item'>E</li>
          <li id='six' class='item'>F</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const odd = document.querySelector('li:nth-last-child(odd)').id;
            const even = document.querySelector('li:nth-last-child(even)').id;
            const second_last = document.querySelector('li:nth-last-child(2)').id;
            const total = document.querySelectorAll('li:nth-last-child(odd)').length;
            document.getElementById('result').textContent = odd + ':' + even + ':' + second_last + ':' + total;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "two:one:five:3")?;
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

    #[test]
    fn disabled_property_prevents_user_actions_and_can_be_cleared() -> Result<()> {
        let html = r#"
        <input id='name' value='init'>
        <input id='agree' type='checkbox' checked>
        <button id='disable'>disable</button>
        <button id='enable'>enable</button>
        <p id='result'></p>
        <script>
          document.getElementById('disable').addEventListener('click', () => {
            document.getElementById('name').disabled = true;
            document.getElementById('agree').disabled = true;
          });
          document.getElementById('enable').addEventListener('click', () => {
            document.getElementById('name').disabled = false;
            document.getElementById('agree').disabled = false;
          });
          document.getElementById('name').addEventListener('input', () => {
            document.getElementById('result').textContent = 'name-input';
          });
          document.getElementById('agree').addEventListener('change', () => {
            document.getElementById('result').textContent = 'agree-change';
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#disable")?;

        h.type_text("#name", "next")?;
        h.assert_value("#name", "init")?;
        h.click("#agree")?;
        h.assert_checked("#agree", true)?;
        h.assert_text("#result", "")?;

        h.click("#enable")?;
        h.type_text("#name", "next")?;
        h.set_checked("#agree", false)?;
        h.assert_value("#name", "next")?;
        h.assert_checked("#agree", false)?;
        Ok(())
    }

    #[test]
    fn assignment_and_remainder_expressions_work() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let n = 20;
            n += 5;
            n -= 3;
            n *= 2;
            n /= 4;
            n %= 6;
            const eq = (10 % 3) == 1;
            const neq = (10 % 3) != 2;
            document.getElementById('result').textContent =
              n + ':' + (eq ? 'eq' : 'neq') + ':' + (neq ? 'neq' : 'eq');
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "5:eq:neq")?;
        Ok(())
    }

    #[test]
    fn unary_plus_works_as_numeric_expression() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const text = '12';
            const value = +text;
            const direct = +'-3.5';
            const paren = +('+7');
            document.getElementById('result').textContent =
              value + ':' + direct + ':' + paren;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "12:-3.5:7")?;
        Ok(())
    }

    #[test]
    fn bitwise_expression_supports_binary_operations() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const bit_and = 5 & 3;
            const bit_or = 5 | 2;
            const bit_xor = 5 ^ 1;
            const left = 1 + 2 << 2;
            const masked = 5 + 2 & 4;
            const shift = 8 >>> 1;
            const signed_shift = -8 >> 1;
            const unsigned_shift = (-1) >>> 1;
            const inv = ~1;
            document.getElementById('result').textContent =
              bit_and + ':' + bit_or + ':' + bit_xor + ':' + left + ':' + masked + ':' +
              shift + ':' + signed_shift + ':' + unsigned_shift + ':' + inv;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "1:7:4:12:4:4:-4:2147483647:-2")?;
        Ok(())
    }

    #[test]
    fn bitwise_compound_assignment_operators_work() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let n = 6;
            n &= 3;
            n |= 4;
            n ^= 1;
            n <<= 1;
            n >>= 1;
            n >>>= 1;
            document.getElementById('result').textContent = n;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "3")?;
        Ok(())
    }

    #[test]
    fn exponentiation_expression_and_compound_assignment_work() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const value = 2 ** 3 ** 2;
            const with_mul = 2 * 3 ** 2;
            const grouped = (2 + 2) ** 3;
            let n = 2;
            n **= 3;
            document.getElementById('result').textContent =
              value + ':' + with_mul + ':' + grouped + ':' + n;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "512:18:64:8")?;
        Ok(())
    }

    #[test]
    fn update_statements_change_identifier_values() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let n = 1;
            ++n;
            n++;
            --n;
            n--;
            document.getElementById('result').textContent = n;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "1")?;
        Ok(())
    }

    #[test]
    fn typeof_operator_works_for_known_and_undefined_identifiers() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const known = 1;
            const a = typeof known;
            const b = typeof unknownName;
            const c = typeof false;
            document.getElementById('result').textContent = a + ':' + b + ':' + c;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "number:undefined:boolean")?;
        Ok(())
    }

    #[test]
    fn undefined_void_delete_and_special_literals_work() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const known = 1;
            const is_void = void known;
            const a = typeof undefined;
            const b = typeof is_void;
            const c = typeof NaN;
            const d = typeof Infinity;
            const e = is_void === undefined;
            const f = delete known;
            const g = delete missing;
            const h = NaN === NaN;
            document.getElementById('result').textContent =
              a + ':' + b + ':' + c + ':' + d + ':' + e + ':' + f + ':' + g + ':' + h;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "undefined:undefined:number:number:true:false:true:false")?;
        Ok(())
    }

    #[test]
    fn in_operator_works_with_query_selector_all_indexes() -> Result<()> {
        let html = r#"
        <div id='a'>A</div>
        <div id='b'>B</div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const nodes = document.querySelectorAll('#a, #b');
            const a = 0 in nodes;
            const b = 1 in nodes;
            const c = 2 in nodes;
            const d = '1' in nodes;
            document.getElementById('result').textContent = a + ':' + b + ':' + c + ':' + d;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "true:true:false:true")?;
        Ok(())
    }

    #[test]
    fn instanceof_operator_works_with_node_membership_and_identity() -> Result<()> {
        let html = r#"
        <div id='a'>A</div>
        <div id='b'>B</div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a_node = document.getElementById('a');
            const b_node = document.getElementById('b');
            const a_only = document.querySelectorAll('#a');
            const same = a_node instanceof a_node;
            const member = a_node instanceof a_only;
            const other = b_node instanceof a_only;
            document.getElementById('result').textContent = same + ':' + member + ':' + other;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "true:true:false")?;
        Ok(())
    }

    #[test]
    fn numeric_literals_support_hex_octal_binary_and_scientific_notation() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const hex = 0x10;
            const oct = 0o10;
            const bin = 0b10;
            const exp = 1e3;
            document.getElementById('result').textContent =
              hex + ':' + oct + ':' + bin + ':' + exp;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "16:8:2:1000")?;
        Ok(())
    }

    #[test]
    fn encode_decode_uri_global_functions_work() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = encodeURI('https://a.example/a b?x=1&y=2#f');
            const b = encodeURIComponent('a b&c=d');
            const c = decodeURI('https://a.example/a%20b?x=1&y=2#f');
            const d = decodeURI('%3Fx%3D1');
            const e = decodeURIComponent('a%20b%26c%3Dd');
            const f = window.encodeURIComponent('x y');
            document.getElementById('result').textContent =
              a + '|' + b + '|' + c + '|' + d + '|' + e + '|' + f;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text(
            "#result",
            "https://a.example/a%20b?x=1&y=2#f|a%20b%26c%3Dd|https://a.example/a b?x=1&y=2#f|%3Fx%3D1|a b&c=d|x%20y",
        )?;
        Ok(())
    }

    #[test]
    fn decode_uri_invalid_sequence_returns_runtime_error_for_decode_uri() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            decodeURIComponent('%E0%A4%A');
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        let err = h.click("#btn").expect_err("decodeURIComponent should fail for malformed input");
        match err {
            Error::ScriptRuntime(msg) => assert!(msg.contains("malformed URI sequence")),
            other => panic!("unexpected decode URI error: {other:?}"),
        }
        Ok(())
    }

    #[test]
    fn escape_and_unescape_global_functions_work() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const kana = unescape('%u3042');
            const escaped = escape('ABC abc +/' + kana);
            const unescaped = unescape(escaped);
            const viaWindow = window.unescape('%u3042%20A');
            const viaWindowEscaped = window.escape('hello world');
            document.getElementById('result').textContent =
              escaped + '|' + unescaped + '|' + viaWindow + '|' + viaWindowEscaped;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text(
            "#result",
            "ABC%20abc%20+/%u3042|ABC abc +/あ|あ A|hello%20world",
        )?;
        Ok(())
    }

    #[test]
    fn window_aliases_for_global_functions_match_direct_calls() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent =
              window.encodeURI('a b?x=1') + '|' + encodeURI('a b?x=1') + '|' +
              window.decodeURIComponent('a%20b%2Bc') + '|' + decodeURIComponent('a%20b%2Bc') + '|' +
              window.unescape(window.escape('A B')) + '|' +
              window.atob(window.btoa('ok')) + '|' +
              window.isNaN('x') + '|' +
              window.isFinite('3') + '|' +
              window.parseInt('11', 2) + '|' +
              window.parseFloat('2.5z');
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "a%20b?x=1|a%20b?x=1|a b+c|a b+c|A B|ok|true|true|3|2.5")?;
        Ok(())
    }

    #[test]
    fn fetch_uses_registered_mock_response_and_records_calls() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const first = fetch('/api/message');
            const second = window.fetch('/api/message');
            document.getElementById('result').textContent = first + ':' + second;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.set_fetch_mock("/api/message", "ok");
        h.click("#btn")?;
        h.assert_text("#result", "ok:ok")?;
        assert_eq!(
            h.take_fetch_calls(),
            vec!["/api/message".to_string(), "/api/message".to_string()]
        );
        Ok(())
    }

    #[test]
    fn fetch_without_mock_returns_runtime_error() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            fetch('/api/missing');
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        let err = h
            .click("#btn")
            .expect_err("fetch without mock should fail with runtime error");
        match err {
            Error::ScriptRuntime(msg) => assert!(msg.contains("fetch mock not found")),
            other => panic!("unexpected fetch error: {other:?}"),
        }
        Ok(())
    }

    #[test]
    fn alert_confirm_prompt_support_mocked_responses() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const accepted = confirm('continue?');
            const name = prompt('name?', 'guest');
            window.alert('hello ' + name);
            document.getElementById('result').textContent = accepted + ':' + name;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.enqueue_confirm_response(true);
        h.enqueue_prompt_response(Some("kazu"));
        h.click("#btn")?;
        h.assert_text("#result", "true:kazu")?;
        assert_eq!(h.take_alert_messages(), vec!["hello kazu".to_string()]);
        Ok(())
    }

    #[test]
    fn prompt_uses_default_argument_when_no_mock_response() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const name = prompt('name?', 'guest');
            document.getElementById('result').textContent = name;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "guest")?;
        Ok(())
    }

    #[test]
    fn global_function_arity_errors_have_stable_messages() {
        let cases = [
            (
                "<script>encodeURI();</script>",
                "encodeURI requires exactly one argument",
            ),
            (
                "<script>window.encodeURIComponent('a', 'b');</script>",
                "encodeURIComponent requires exactly one argument",
            ),
            (
                "<script>decodeURI('a', 'b');</script>",
                "decodeURI requires exactly one argument",
            ),
            (
                "<script>window.decodeURIComponent();</script>",
                "decodeURIComponent requires exactly one argument",
            ),
            (
                "<script>escape();</script>",
                "escape requires exactly one argument",
            ),
            (
                "<script>window.unescape('a', 'b');</script>",
                "unescape requires exactly one argument",
            ),
            (
                "<script>isNaN();</script>",
                "isNaN requires exactly one argument",
            ),
            (
                "<script>window.isFinite();</script>",
                "isFinite requires exactly one argument",
            ),
            (
                "<script>atob('YQ==', 'x');</script>",
                "atob requires exactly one argument",
            ),
            (
                "<script>window.btoa();</script>",
                "btoa requires exactly one argument",
            ),
            (
                "<script>parseFloat('1', 10);</script>",
                "parseFloat requires exactly one argument",
            ),
            (
                "<script>window.parseInt('1', 10, 10);</script>",
                "parseInt requires one or two arguments",
            ),
            (
                "<script>fetch();</script>",
                "fetch requires exactly one argument",
            ),
            (
                "<script>alert();</script>",
                "alert requires exactly one argument",
            ),
            (
                "<script>window.confirm('ok', 'ng');</script>",
                "confirm requires exactly one argument",
            ),
            (
                "<script>prompt();</script>",
                "prompt requires one or two arguments",
            ),
            (
                "<script>window.prompt('x', );</script>",
                "prompt default argument cannot be empty",
            ),
            (
                "<script>Array.isArray();</script>",
                "Array.isArray requires exactly one argument",
            ),
        ];

        for (html, expected) in cases {
            let err = Harness::from_html(html).expect_err("script should fail to parse");
            match err {
                Error::ScriptParse(msg) => assert!(
                    msg.contains(expected),
                    "expected '{expected}' in '{msg}'"
                ),
                other => panic!("unexpected error: {other:?}"),
            }
        }
    }

    #[test]
    fn global_function_parser_respects_identifier_boundaries() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const escaped = escape('A B');
            const encodedValue = encodeURIComponent('x y');
            const parseIntValue = 7;
            const parseFloatValue = 1.25;
            const escapedValue = escaped;
            const round = unescape(escapedValue);
            document.getElementById('result').textContent =
              escapedValue + ':' + encodedValue + ':' + round + ':' +
              parseIntValue + ':' + parseFloatValue;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "A%20B:x%20y:A B:7:1.25")?;
        Ok(())
    }

    #[test]
    fn btoa_non_latin1_input_returns_runtime_error() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const nonLatin1 = unescape('%u3042');
            btoa(nonLatin1);
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        let err = h
            .click("#btn")
            .expect_err("btoa should reject non-Latin1 input");
        match err {
            Error::ScriptRuntime(msg) => assert!(msg.contains("non-Latin1")),
            other => panic!("unexpected btoa error: {other:?}"),
        }
        Ok(())
    }

    #[test]
    fn decode_uri_invalid_sequence_returns_runtime_error() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            decodeURI('%E0%A4%A');
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        let err = h.click("#btn").expect_err("decodeURI should fail for malformed input");
        match err {
            Error::ScriptRuntime(msg) => assert!(msg.contains("malformed URI sequence")),
            other => panic!("unexpected decode URI error: {other:?}"),
        }
        Ok(())
    }

    #[test]
    fn is_nan_and_is_finite_global_functions_work() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = isNaN('abc');
            const b = isNaN('  ');
            const c = isNaN(undefined);
            const d = isFinite('1.5');
            const e = isFinite(Infinity);
            const f = window.isFinite(null);
            const g = window.isNaN(NaN);
            document.getElementById('result').textContent =
              a + ':' + b + ':' + c + ':' + d + ':' + e + ':' + f + ':' + g;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "true:false:true:true:false:true:true")?;
        Ok(())
    }

    #[test]
    fn atob_and_btoa_global_functions_work() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const encoded = btoa('abc123!?');
            const decoded = atob(encoded);
            const viaWindow = window.atob('QQ==');
            document.getElementById('result').textContent =
              encoded + ':' + decoded + ':' + viaWindow;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "YWJjMTIzIT8=:abc123!?:A")?;
        Ok(())
    }

    #[test]
    fn atob_invalid_input_returns_runtime_error() -> Result<()> {
        let html = r#"
        <button id='atob'>atob</button>
        <script>
          document.getElementById('atob').addEventListener('click', () => {
            atob('@@@');
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;

        let atob_err = h.click("#atob").expect_err("atob should reject invalid base64");
        match atob_err {
            Error::ScriptRuntime(msg) => assert!(msg.contains("invalid base64")),
            other => panic!("unexpected atob error: {other:?}"),
        }

        Ok(())
    }

    #[test]
    fn parse_int_global_function_works() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = parseInt('42px');
            const b = parseInt('  -0x10');
            const c = parseInt('10', 2);
            const d = parseInt('10', 8);
            const e = parseInt('0x10', 16);
            const bad1 = parseInt('xyz');
            const bad2 = parseInt('10', 1);
            const f = window.parseInt('12', 10);
            document.getElementById('result').textContent =
              a + ':' + b + ':' + c + ':' + d + ':' + e + ':' +
              (bad1 === bad1) + ':' + (bad2 === bad2) + ':' + f;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "42:-16:2:8:16:false:false:12")?;
        Ok(())
    }

    #[test]
    fn parse_float_global_function_works() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = parseFloat('3.5px');
            const b = parseFloat('  -2.5e2x');
            const invalid = parseFloat('abc');
            const d = window.parseFloat('42');
            document.getElementById('result').textContent =
              a + ':' + b + ':' + (invalid === invalid) + ':' + d;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "3.5:-250:false:42")?;
        Ok(())
    }

    #[test]
    fn array_literal_and_basic_mutation_methods_work() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const arr = [1, 2];
            const isArray1 = Array.isArray(arr);
            const isArray2 = window.Array.isArray('x');
            const lenBefore = arr.length;
            const first = arr[0];
            const pushed = arr.push(3, 4);
            const popped = arr.pop();
            const shifted = arr.shift();
            const unshifted = arr.unshift(9);
            document.getElementById('result').textContent =
              isArray1 + ':' + isArray2 + ':' + lenBefore + ':' + first + ':' +
              pushed + ':' + popped + ':' + shifted + ':' + unshifted + ':' + arr.join(',');
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "true:false:2:1:4:4:1:3:9,2,3")?;
        Ok(())
    }

    #[test]
    fn array_map_filter_and_reduce_work() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const arr = [1, 2, 3, 4];
            const mapped = arr.map((value, index) => value * 2 + index);
            const filtered = mapped.filter(value => value > 5);
            const sum = filtered.reduce((acc, value) => acc + value, 0);
            const sumNoInitial = filtered.reduce((acc, value) => acc + value);
            document.getElementById('result').textContent =
              mapped.join(',') + '|' + filtered.join(',') + '|' + sum + '|' + sumNoInitial;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "2,5,8,11|8,11|19|19")?;
        Ok(())
    }

    #[test]
    fn array_foreach_find_some_every_and_includes_work() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const arr = [2, 4, 6];
            let total = 0;
            arr.forEach((value, idx) => {
              total += value + idx;
            });
            const found = arr.find(value => value > 3);
            const some = arr.some(value => value === 4);
            const every = arr.every(value => value % 2 === 0);
            const includesDirect = arr.includes(4);
            const includesFrom = arr.includes(2, 1);
            document.getElementById('result').textContent =
              total + ':' + found + ':' + some + ':' + every + ':' +
              includesDirect + ':' + includesFrom;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "15:4:true:true:true:false")?;
        Ok(())
    }

    #[test]
    fn array_slice_splice_and_join_work() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const arr = [1, 2, 3, 4];
            const firstSlice = arr.slice(1, 3);
            const secondSlice = arr.slice(-2);
            const removed = arr.splice(1, 2, 9, 8);
            document.getElementById('result').textContent =
              firstSlice.join(',') + '|' + secondSlice.join(',') + '|' +
              removed.join(',') + '|' + arr.join('-');
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "2,3|3,4|2,3|1-9-8-4")?;
        Ok(())
    }

    #[test]
    fn reduce_empty_array_without_initial_value_returns_runtime_error() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const arr = [];
            arr.reduce((acc, value) => acc + value);
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        let err = h
            .click("#btn")
            .expect_err("reduce without initial on empty array should fail");
        match err {
            Error::ScriptRuntime(msg) => {
                assert!(msg.contains("reduce of empty array with no initial value"))
            }
            other => panic!("unexpected reduce error: {other:?}"),
        }
        Ok(())
    }

    #[test]
    fn string_trim_and_case_methods_work() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const raw = '  AbC ';
            const trimmed = raw.trim();
            const trimmedStart = raw.trimStart();
            const trimmedEnd = raw.trimEnd();
            const upper = raw.toUpperCase();
            const lower = raw.toLowerCase();
            const literal = ' z '.trim();
            document.getElementById('result').textContent =
              '[' + trimmed + ']|[' + trimmedStart + ']|[' + trimmedEnd + ']|[' +
              upper + ']|[' + lower + ']|[' + literal + ']';
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "[AbC]|[AbC ]|[  AbC]|[  ABC ]|[  abc ]|[z]")?;
        Ok(())
    }

    #[test]
    fn string_includes_prefix_suffix_and_index_methods_work() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const text = 'hello world';
            const includes1 = text.includes('lo');
            const includes2 = text.includes('lo', 4);
            const includes3 = 'abc'.includes('a', -2);
            const starts1 = text.startsWith('hello');
            const starts2 = text.startsWith('world', 6);
            const starts3 = 'abc'.startsWith('a');
            const ends1 = text.endsWith('world');
            const ends2 = text.endsWith('hello', 5);
            const index1 = text.indexOf('o');
            const index2 = text.indexOf('o', 5);
            const index3 = text.indexOf('x');
            const index4 = text.indexOf('', 2);
            document.getElementById('result').textContent =
              includes1 + ':' + includes2 + ':' + includes3 + ':' +
              starts1 + ':' + starts2 + ':' + starts3 + ':' +
              ends1 + ':' + ends2 + ':' +
              index1 + ':' + index2 + ':' + index3 + ':' + index4;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "true:false:true:true:true:true:true:true:4:7:-1:2")?;
        Ok(())
    }

    #[test]
    fn string_slice_substring_split_and_replace_work() -> Result<()> {
        let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const text = '012345';
            const s1 = text.slice(1, 4);
            const s2 = text.slice(-2);
            const s3 = text.slice(4, 1);
            const sub1 = text.substring(1, 4);
            const sub2 = text.substring(4, 1);
            const sub3 = text.substring(-2, 2);
            const split1 = 'a,b,c'.split(',');
            const split2 = 'abc'.split('');
            const split3 = 'a,b,c'.split(',', 2);
            const split4 = 'abc'.split();
            const rep1 = 'foo foo'.replace('foo', 'bar');
            const rep2 = 'abc'.replace('', '-');
            document.getElementById('result').textContent =
              s1 + ':' + s2 + ':' + s3.length + ':' +
              sub1 + ':' + sub2 + ':' + sub3 + ':' +
              split1.join('-') + ':' + split2.join('|') + ':' + split3.join(':') + ':' +
              split4.length + ':' + rep1 + ':' + rep2;
          });
        </script>
        "#;

        let mut h = Harness::from_html(html)?;
        h.click("#btn")?;
        h.assert_text("#result", "123:45:0:123:123:01:a-b-c:a|b|c:a:b:1:bar foo:-abc")?;
        Ok(())
    }

    #[test]
    fn string_method_arity_errors_have_stable_messages() {
        let cases = [
            ("<script>'x'.trim(1);</script>", "trim does not take arguments"),
            (
                "<script>'x'.toUpperCase(1);</script>",
                "toUpperCase does not take arguments",
            ),
            (
                "<script>'x'.includes();</script>",
                "String.includes requires one or two arguments",
            ),
            (
                "<script>'x'.startsWith();</script>",
                "startsWith requires one or two arguments",
            ),
            (
                "<script>'x'.endsWith();</script>",
                "endsWith requires one or two arguments",
            ),
            (
                "<script>'x'.slice(, 1);</script>",
                "String.slice start cannot be empty",
            ),
            (
                "<script>'x'.substring(, 1);</script>",
                "substring start cannot be empty",
            ),
            (
                "<script>'x'.split(, 1);</script>",
                "split separator cannot be empty expression",
            ),
            (
                "<script>'x'.replace('a');</script>",
                "replace requires exactly two arguments",
            ),
            (
                "<script>'x'.indexOf();</script>",
                "indexOf requires one or two arguments",
            ),
        ];

        for (html, expected) in cases {
            let err = Harness::from_html(html).expect_err("script should fail to parse");
            match err {
                Error::ScriptParse(msg) => assert!(
                    msg.contains(expected),
                    "expected '{expected}' in '{msg}'"
                ),
                other => panic!("unexpected error: {other:?}"),
            }
        }
    }
}
