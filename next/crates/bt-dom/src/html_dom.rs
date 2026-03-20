use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fmt::Write as _;

use super::DomIndexes;
use super::DomStore;
use super::ElementData;
use super::HTML_NAMESPACE_URI;
use super::MATHML_NAMESPACE_URI;
use super::NodeId;
use super::NodeKind;
use super::NodeRecord;
use super::SVG_NAMESPACE_URI;
use super::TextData;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct SelectorQuery {
    tag: Option<String>,
    id: Option<String>,
    classes: Vec<String>,
    attributes: Vec<SelectorAttribute>,
    pseudo_classes: Vec<SelectorPseudoClass>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SelectorAttribute {
    name: String,
    operator: SelectorAttributeOperator,
    value: Option<String>,
    case_sensitivity: SelectorAttributeCaseSensitivity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SelectorAttributeOperator {
    Exists,
    Exact,
    Prefix,
    Suffix,
    Contains,
    Includes,
    DashMatch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SelectorAttributeCaseSensitivity {
    CaseSensitive,
    AsciiInsensitive,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct SelectorChain {
    parts: Vec<SelectorQuery>,
    relations: Vec<SelectorCombinator>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SelectorCombinator {
    Descendant,
    Child,
    AdjacentSibling,
    GeneralSibling,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum SelectorPseudoClass {
    Root,
    Empty,
    OnlyChild,
    OnlyOfType,
    FirstChild,
    LastChild,
    FirstOfType,
    LastOfType,
    NthChild(SelectorNthChildPattern),
    NthLastChild(SelectorNthChildPattern),
    NthOfType(SelectorNthChildPattern),
    NthLastOfType(SelectorNthChildPattern),
    Is(Vec<SelectorChain>),
    Where(Vec<SelectorChain>),
    Not(Vec<SelectorChain>),
    Checked,
    Disabled,
    Enabled,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SelectorNthChildPattern {
    step: isize,
    offset: isize,
}

impl DomStore {
    pub fn bootstrap_html(&mut self, html: impl Into<String>) -> Result<(), String> {
        let html = html.into();
        let mut parsed = Self::new_empty();
        parsed.source_html = Some(html.clone());
        let mut parser = HtmlParser::new(&html);
        parser.parse_into(&mut parsed)?;
        parsed.rebuild_form_controls();
        *self = parsed;
        Ok(())
    }

    pub fn select(&self, selector: &str) -> Result<Vec<NodeId>, String> {
        let selector = selector.trim();
        if selector.is_empty() {
            return Err("selector must not be empty".to_string());
        }

        let chains = Self::parse_selector_list(selector)?;
        Ok(self.select_by_selector_chains(&chains))
    }

    pub fn dump_dom(&self) -> String {
        let mut output = String::new();
        self.dump_node(self.document_id, 0, &mut output);
        output
    }

    pub fn set_text_content(&mut self, node_id: NodeId, value: &str) -> Result<(), String> {
        let node_index = node_id.index() as usize;
        let old_children = {
            let Some(node) = self.nodes.get_mut(node_index) else {
                return Err(format!("invalid node id: {:?}", node_id));
            };

            match &mut node.kind {
                NodeKind::Document => return Ok(()),
                NodeKind::Text(text) => {
                    text.value = value.to_string();
                    return Ok(());
                }
                NodeKind::Comment(comment) => {
                    comment.clear();
                    comment.push_str(value);
                    return Ok(());
                }
                NodeKind::Element(_) => std::mem::take(&mut node.children),
            }
        };

        let removed_nodes = self.collect_subtree_nodes(old_children.iter().copied());
        for removed_id in removed_nodes {
            if let Some(record) = self.nodes.get_mut(removed_id.index() as usize) {
                record.parent = None;
            }
            self.side_tables.form_controls.remove(&removed_id);
            self.side_tables.selection.remove(&removed_id);
            self.side_tables.file_inputs.remove(&removed_id);
            self.side_tables.dialogs.remove(&removed_id);
            self.side_tables.layout_stub.remove(&removed_id);
        }

        if !value.is_empty() {
            self.add_text(node_id, value.to_string());
        }

        self.rebuild_indexes();
        self.rebuild_form_controls();
        Ok(())
    }

    pub fn text_content_for_node(&self, node_id: NodeId) -> String {
        let Some(node) = self.nodes.get(node_id.index() as usize) else {
            return String::new();
        };

        match &node.kind {
            NodeKind::Document | NodeKind::Element(_) => {
                let mut out = String::new();
                for child in &node.children {
                    out.push_str(&self.text_content_for_node(*child));
                }
                out
            }
            NodeKind::Text(text) => text.value.clone(),
            NodeKind::Comment(_) => String::new(),
        }
    }

    pub fn value_for_node(&self, node_id: NodeId) -> String {
        if let Some(state) = self.side_tables.form_controls.get(&node_id) {
            return state.value.clone();
        }

        let Some(node) = self.nodes.get(node_id.index() as usize) else {
            return String::new();
        };

        match &node.kind {
            NodeKind::Element(element) if element.tag_name == "select" => {
                self.select_value_for_node(node_id)
            }
            NodeKind::Element(element) if element.tag_name == "option" => {
                self.option_value_for_node(node_id)
            }
            NodeKind::Element(element)
                if element.tag_name == "input"
                    && is_file_input_type(element.attributes.get("type").map(String::as_str)) =>
            {
                self.file_input_value_for_node(node_id)
            }
            NodeKind::Element(element) => element
                .attributes
                .get("value")
                .cloned()
                .unwrap_or_else(|| self.text_content_for_node(node_id)),
            NodeKind::Document => self.text_content_for_node(node_id),
            NodeKind::Text(text) => text.value.clone(),
            NodeKind::Comment(_) => String::new(),
        }
    }

    pub fn checked_for_node(&self, node_id: NodeId) -> Option<bool> {
        let Some(node) = self.nodes.get(node_id.index() as usize) else {
            return None;
        };

        let NodeKind::Element(element) = &node.kind else {
            return None;
        };

        if element.tag_name == "input"
            && is_checkable_input_type(element.attributes.get("type").map(String::as_str))
        {
            self.side_tables
                .form_controls
                .get(&node_id)
                .map(|state| state.checked)
                .or_else(|| Some(element.attributes.contains_key("checked")))
        } else {
            None
        }
    }

    pub fn set_form_control_value(
        &mut self,
        node_id: NodeId,
        value: impl Into<String>,
    ) -> Result<(), String> {
        let value = value.into();
        let node_index = node_id.index() as usize;
        let Some(node) = self.nodes.get(node_index) else {
            return Err(format!("invalid node id: {:?}", node_id));
        };

        let NodeKind::Element(element) = &node.kind else {
            return Err(format!(
                "node {:?} is not a supported form control",
                node_id
            ));
        };

        match element.tag_name.as_str() {
            "textarea" => self.set_text_content(node_id, &value),
            "input" if is_text_input_type(element.attributes.get("type").map(String::as_str)) => {
                {
                    let Some(node) = self.nodes.get_mut(node_index) else {
                        return Err(format!("invalid node id: {:?}", node_id));
                    };
                    let NodeKind::Element(element) = &mut node.kind else {
                        return Err(format!(
                            "node {:?} is not a supported form control",
                            node_id
                        ));
                    };
                    element
                        .attributes
                        .insert("value".to_string(), value.clone());
                }
                self.rebuild_form_controls();
                Ok(())
            }
            "input" => Err(format!(
                "set_value is only supported on text-like inputs and textareas, not <input type=\"{}\">",
                element
                    .attributes
                    .get("type")
                    .map(String::as_str)
                    .unwrap_or("text")
            )),
            _ => Err(format!(
                "node {:?} is not a supported form control",
                node_id
            )),
        }
    }

    pub fn set_select_value(
        &mut self,
        node_id: NodeId,
        value: impl Into<String>,
    ) -> Result<(), String> {
        let value = value.into();
        let node_index = node_id.index() as usize;
        let Some(node) = self.nodes.get(node_index) else {
            return Err(format!("invalid node id: {:?}", node_id));
        };

        let NodeKind::Element(element) = &node.kind else {
            return Err(format!("node {:?} is not a select control", node_id));
        };

        if element.tag_name != "select" {
            return Err(format!("node {:?} is not a select control", node_id));
        }

        let option_ids = self.collect_subtree_nodes(node.children.iter().copied());
        let options: Vec<NodeId> = option_ids
            .into_iter()
            .filter(|option_id| self.is_option_node(*option_id))
            .collect();

        if options.is_empty() {
            return Err(format!(
                "select node {:?} does not contain any options",
                node_id
            ));
        }

        let mut found = false;
        for option_id in &options {
            let option_value = self.option_value_for_node(*option_id);
            if option_value == value {
                found = true;
                break;
            }
        }

        if !found {
            return Err(format!(
                "select node {:?} does not contain an option with value `{}`",
                node_id, value
            ));
        }

        for option_id in options {
            let selected = self.option_value_for_node(option_id) == value;
            self.set_option_selected(option_id, selected)?;
        }

        Ok(())
    }

    pub fn set_form_control_checked(
        &mut self,
        node_id: NodeId,
        checked: bool,
    ) -> Result<(), String> {
        let node_index = node_id.index() as usize;
        let Some(node) = self.nodes.get(node_index) else {
            return Err(format!("invalid node id: {:?}", node_id));
        };

        let NodeKind::Element(element) = &node.kind else {
            return Err(format!(
                "node {:?} is not a supported form control",
                node_id
            ));
        };

        match element.tag_name.as_str() {
            "input"
                if is_checkable_input_type(element.attributes.get("type").map(String::as_str)) =>
            {
                {
                    let Some(node) = self.nodes.get_mut(node_index) else {
                        return Err(format!("invalid node id: {:?}", node_id));
                    };
                    let NodeKind::Element(element) = &mut node.kind else {
                        return Err(format!(
                            "node {:?} is not a supported form control",
                            node_id
                        ));
                    };
                    if checked {
                        element
                            .attributes
                            .insert("checked".to_string(), String::new());
                    } else {
                        element.attributes.remove("checked");
                    }
                }
                self.rebuild_form_controls();
                Ok(())
            }
            "input" => Err(format!(
                "set_checked is only supported on checkbox and radio inputs, not <input type=\"{}\">",
                element
                    .attributes
                    .get("type")
                    .map(String::as_str)
                    .unwrap_or("text")
            )),
            _ => Err(format!(
                "node {:?} is not a supported form control",
                node_id
            )),
        }
    }

    pub fn set_file_input_files(
        &mut self,
        node_id: NodeId,
        files: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<(), String> {
        let node_index = node_id.index() as usize;
        let Some(node) = self.nodes.get(node_index) else {
            return Err(format!("invalid node id: {:?}", node_id));
        };

        let NodeKind::Element(element) = &node.kind else {
            return Err(format!("node {:?} is not a file input control", node_id));
        };

        if element.tag_name != "input"
            || !is_file_input_type(element.attributes.get("type").map(String::as_str))
        {
            return Err(format!("node {:?} is not a file input control", node_id));
        }

        self.side_tables.file_inputs.insert(
            node_id,
            super::FileInputState {
                files: files.into_iter().map(Into::into).collect(),
            },
        );
        Ok(())
    }

    fn add_node(&mut self, parent: NodeId, kind: NodeKind) -> NodeId {
        let id = NodeId::new(self.nodes.len() as u32, 0);
        self.nodes.push(NodeRecord {
            id,
            parent: Some(parent),
            children: Vec::new(),
            kind,
        });
        self.nodes[parent.index() as usize].children.push(id);
        id
    }

    fn add_element(
        &mut self,
        parent: NodeId,
        tag_name: String,
        attributes: BTreeMap<String, String>,
    ) -> NodeId {
        let namespace_uri = self.namespace_uri_for_child(parent, &tag_name).to_string();
        let node_id = self.add_node(
            parent,
            NodeKind::Element(ElementData {
                tag_name: tag_name.clone(),
                local_name: tag_name.clone(),
                namespace_uri,
                attributes: attributes.clone(),
            }),
        );

        self.indexes
            .tag_index
            .entry(tag_name.clone())
            .or_default()
            .push(node_id);

        if let Some(value) = attributes.get("id") {
            self.indexes
                .id_index
                .entry(value.clone())
                .or_insert(node_id);
        }

        if let Some(value) = attributes.get("name") {
            self.indexes
                .name_index
                .entry(value.clone())
                .or_default()
                .push(node_id);
        }

        if let Some(value) = attributes.get("class") {
            for class_name in value.split_ascii_whitespace() {
                if !class_name.is_empty() {
                    self.indexes
                        .class_index
                        .entry(class_name.to_string())
                        .or_default()
                        .push(node_id);
                }
            }
        }

        node_id
    }

    fn namespace_uri_for_child(&self, parent: NodeId, tag_name: &str) -> &'static str {
        let parent_kind = self
            .nodes
            .get(parent.index() as usize)
            .map(|node| &node.kind);
        let parent_element = match parent_kind {
            Some(NodeKind::Element(element)) => Some(element),
            _ => None,
        };

        match parent_element {
            None => element_namespace_for_root(tag_name),
            Some(element)
                if element.namespace_uri == SVG_NAMESPACE_URI
                    && element.local_name == "foreignobject" =>
            {
                element_namespace_for_root(tag_name)
            }
            Some(element) if element.namespace_uri == SVG_NAMESPACE_URI => SVG_NAMESPACE_URI,
            Some(element) if element.namespace_uri == MATHML_NAMESPACE_URI => MATHML_NAMESPACE_URI,
            Some(_) => element_namespace_for_root(tag_name),
        }
    }

    fn add_text(&mut self, parent: NodeId, value: String) -> NodeId {
        self.add_node(parent, NodeKind::Text(TextData { value }))
    }

    fn add_comment(&mut self, parent: NodeId, value: String) -> NodeId {
        self.add_node(parent, NodeKind::Comment(value))
    }

    fn is_option_node(&self, node_id: NodeId) -> bool {
        matches!(
            self.nodes.get(node_id.index() as usize).map(|node| &node.kind),
            Some(NodeKind::Element(element)) if element.tag_name == "option"
        )
    }

    fn option_value_for_node(&self, node_id: NodeId) -> String {
        let Some(node) = self.nodes.get(node_id.index() as usize) else {
            return String::new();
        };

        let NodeKind::Element(element) = &node.kind else {
            return String::new();
        };

        element
            .attributes
            .get("value")
            .cloned()
            .unwrap_or_else(|| self.text_content_for_node(node_id))
    }

    fn file_input_value_for_node(&self, node_id: NodeId) -> String {
        self.side_tables
            .file_inputs
            .get(&node_id)
            .map(|state| state.files.join(", "))
            .unwrap_or_default()
    }

    fn set_option_selected(&mut self, node_id: NodeId, selected: bool) -> Result<(), String> {
        let node_index = node_id.index() as usize;
        let Some(node) = self.nodes.get_mut(node_index) else {
            return Err(format!("invalid node id: {:?}", node_id));
        };

        let NodeKind::Element(element) = &mut node.kind else {
            return Err(format!("node {:?} is not an option element", node_id));
        };

        if element.tag_name != "option" {
            return Err(format!("node {:?} is not an option element", node_id));
        }

        if selected {
            element
                .attributes
                .insert("selected".to_string(), String::new());
        } else {
            element.attributes.remove("selected");
        }

        Ok(())
    }

    fn select_value_for_node(&self, node_id: NodeId) -> String {
        let Some(node) = self.nodes.get(node_id.index() as usize) else {
            return String::new();
        };

        let NodeKind::Element(element) = &node.kind else {
            return String::new();
        };

        if element.tag_name != "select" {
            return String::new();
        }

        let descendants = self.collect_subtree_nodes(node.children.iter().copied());
        let mut first_option_value: Option<String> = None;
        for descendant_id in descendants {
            if !self.is_option_node(descendant_id) {
                continue;
            }

            let option_value = self.option_value_for_node(descendant_id);
            if first_option_value.is_none() {
                first_option_value = Some(option_value.clone());
            }

            if self.is_option_selected(descendant_id) {
                return option_value;
            }
        }

        first_option_value.unwrap_or_default()
    }

    fn is_option_selected(&self, node_id: NodeId) -> bool {
        let Some(node) = self.nodes.get(node_id.index() as usize) else {
            return false;
        };

        let NodeKind::Element(element) = &node.kind else {
            return false;
        };

        element.attributes.contains_key("selected")
    }

    fn collect_subtree_nodes<I>(&self, roots: I) -> Vec<NodeId>
    where
        I: IntoIterator<Item = NodeId>,
    {
        let mut collected = Vec::new();
        for root in roots {
            self.collect_subtree_nodes_inner(root, &mut collected);
        }
        collected
    }

    fn collect_subtree_nodes_inner(&self, node_id: NodeId, collected: &mut Vec<NodeId>) {
        collected.push(node_id);
        let Some(node) = self.nodes.get(node_id.index() as usize) else {
            return;
        };
        for child in &node.children {
            self.collect_subtree_nodes_inner(*child, collected);
        }
    }

    fn rebuild_form_controls(&mut self) {
        self.side_tables.form_controls.clear();
        self.index_form_controls(self.document_id);
    }

    fn index_form_controls(&mut self, node_id: NodeId) {
        let Some(node) = self.nodes.get(node_id.index() as usize).cloned() else {
            return;
        };

        if let NodeKind::Element(element) = &node.kind {
            match element.tag_name.as_str() {
                "textarea" => {
                    self.side_tables.form_controls.insert(
                        node_id,
                        super::FormControlState {
                            value: self.text_content_for_node(node_id),
                            checked: false,
                        },
                    );
                }
                "input"
                    if is_text_input_type(element.attributes.get("type").map(String::as_str)) =>
                {
                    self.side_tables.form_controls.insert(
                        node_id,
                        super::FormControlState {
                            value: element.attributes.get("value").cloned().unwrap_or_default(),
                            checked: false,
                        },
                    );
                }
                "input"
                    if is_checkable_input_type(
                        element.attributes.get("type").map(String::as_str),
                    ) =>
                {
                    self.side_tables.form_controls.insert(
                        node_id,
                        super::FormControlState {
                            value: element
                                .attributes
                                .get("value")
                                .cloned()
                                .unwrap_or_else(|| "on".to_string()),
                            checked: element.attributes.contains_key("checked"),
                        },
                    );
                }
                _ => {}
            }
        }

        for child in node.children {
            self.index_form_controls(child);
        }
    }

    fn rebuild_indexes(&mut self) {
        self.indexes = DomIndexes::default();
        self.index_node(self.document_id);
    }

    fn index_node(&mut self, node_id: NodeId) {
        let Some(node) = self.nodes.get(node_id.index() as usize).cloned() else {
            return;
        };

        if let NodeKind::Element(element) = node.kind {
            self.indexes
                .tag_index
                .entry(element.tag_name.clone())
                .or_default()
                .push(node_id);

            if let Some(value) = element.attributes.get("id") {
                self.indexes
                    .id_index
                    .entry(value.clone())
                    .or_insert(node_id);
            }

            if let Some(value) = element.attributes.get("name") {
                self.indexes
                    .name_index
                    .entry(value.clone())
                    .or_default()
                    .push(node_id);
            }

            if let Some(value) = element.attributes.get("class") {
                for class_name in value.split_ascii_whitespace() {
                    if !class_name.is_empty() {
                        self.indexes
                            .class_index
                            .entry(class_name.to_string())
                            .or_default()
                            .push(node_id);
                    }
                }
            }
        }

        for child in node.children {
            self.index_node(child);
        }
    }

    fn select_by_chain(&self, chain: &SelectorChain) -> Vec<NodeId> {
        let Some(last) = chain.parts.last() else {
            return Vec::new();
        };

        let candidates = self.selector_candidates(last);
        let mut results: Vec<NodeId> = candidates
            .into_iter()
            .filter(|node_id| self.matches_selector_chain(*node_id, chain))
            .collect();
        results.dedup();
        results
    }

    fn select_by_selector_chains(&self, chains: &[SelectorChain]) -> Vec<NodeId> {
        match chains {
            [] => Vec::new(),
            [single] => self.select_by_chain(single),
            _ => {
                let mut matched = BTreeSet::new();
                for chain in chains {
                    matched.extend(self.select_by_chain(chain));
                }

                self.nodes
                    .iter()
                    .filter_map(|node| match &node.kind {
                        NodeKind::Element(_) if matched.contains(&node.id) => Some(node.id),
                        _ => None,
                    })
                    .collect()
            }
        }
    }

    fn selector_candidates(&self, query: &SelectorQuery) -> Vec<NodeId> {
        if let Some(id) = query.id.as_ref() {
            return self.indexes.id_index.get(id).copied().into_iter().collect();
        }

        let mut candidate_lists: Vec<&[NodeId]> = Vec::new();

        if let Some(tag) = query.tag.as_ref() {
            match self.indexes.tag_index.get(tag) {
                Some(nodes) => candidate_lists.push(nodes),
                None => return Vec::new(),
            }
        }

        for class_name in &query.classes {
            match self.indexes.class_index.get(class_name) {
                Some(nodes) => candidate_lists.push(nodes),
                None => return Vec::new(),
            }
        }

        if candidate_lists.is_empty() {
            return self
                .nodes
                .iter()
                .filter_map(|node| match node.kind {
                    NodeKind::Element(_) => Some(node.id),
                    _ => None,
                })
                .collect();
        }

        candidate_lists
            .into_iter()
            .min_by_key(|nodes| nodes.len())
            .map(|nodes| nodes.to_vec())
            .unwrap_or_default()
    }

    fn matches_selector_chain(&self, node_id: NodeId, chain: &SelectorChain) -> bool {
        let Some(last_index) = chain.parts.len().checked_sub(1) else {
            return false;
        };
        self.matches_selector_chain_part(node_id, &chain.parts, &chain.relations, last_index)
    }

    fn matches_selector_chain_part(
        &self,
        node_id: NodeId,
        parts: &[SelectorQuery],
        relations: &[SelectorCombinator],
        index: usize,
    ) -> bool {
        if !self.matches_selector_query(node_id, &parts[index]) {
            return false;
        }

        if index == 0 {
            return true;
        }

        match relations[index - 1] {
            SelectorCombinator::Child => {
                let Some(parent_id) = self.parent_of(node_id) else {
                    return false;
                };
                self.matches_selector_chain_part(parent_id, parts, relations, index - 1)
            }
            SelectorCombinator::AdjacentSibling => {
                let Some(previous_sibling) = self.previous_element_sibling_of(node_id) else {
                    return false;
                };
                self.matches_selector_chain_part(previous_sibling, parts, relations, index - 1)
            }
            SelectorCombinator::GeneralSibling => {
                let mut sibling = self.previous_element_sibling_of(node_id);
                while let Some(previous_sibling) = sibling {
                    if self.matches_selector_chain_part(
                        previous_sibling,
                        parts,
                        relations,
                        index - 1,
                    ) {
                        return true;
                    }
                    sibling = self.previous_element_sibling_of(previous_sibling);
                }
                false
            }
            SelectorCombinator::Descendant => {
                let mut ancestor = self.parent_of(node_id);
                while let Some(ancestor_id) = ancestor {
                    if self.matches_selector_chain_part(ancestor_id, parts, relations, index - 1) {
                        return true;
                    }
                    ancestor = self.parent_of(ancestor_id);
                }
                false
            }
        }
    }

    fn matches_selector_query(&self, node_id: NodeId, query: &SelectorQuery) -> bool {
        let Some(node) = self.nodes.get(node_id.index() as usize) else {
            return false;
        };

        let NodeKind::Element(element) = &node.kind else {
            return false;
        };

        if let Some(tag) = query.tag.as_ref() {
            if element.tag_name != *tag {
                return false;
            }
        }

        if let Some(id) = query.id.as_ref() {
            if element.attributes.get("id") != Some(id) {
                return false;
            }
        }

        if !query.classes.is_empty() {
            let Some(value) = element.attributes.get("class") else {
                return false;
            };

            let element_classes: Vec<&str> = value.split_ascii_whitespace().collect();
            if !query.classes.iter().all(|class_name| {
                element_classes
                    .iter()
                    .any(|candidate| candidate == class_name)
            }) {
                return false;
            }
        }

        for attribute in &query.attributes {
            let Some(element_value) = element.attributes.get(&attribute.name) else {
                return false;
            };

            match (
                attribute.operator,
                attribute.value.as_ref(),
                attribute.case_sensitivity,
            ) {
                (SelectorAttributeOperator::Exists, None, _) => {}
                (
                    SelectorAttributeOperator::Exact,
                    Some(value),
                    SelectorAttributeCaseSensitivity::CaseSensitive,
                ) if element_value == value => {}
                (
                    SelectorAttributeOperator::Exact,
                    Some(value),
                    SelectorAttributeCaseSensitivity::AsciiInsensitive,
                ) if element_value.eq_ignore_ascii_case(value) => {}
                (
                    SelectorAttributeOperator::Prefix,
                    Some(value),
                    SelectorAttributeCaseSensitivity::CaseSensitive,
                ) if element_value.starts_with(value) => {}
                (
                    SelectorAttributeOperator::Prefix,
                    Some(value),
                    SelectorAttributeCaseSensitivity::AsciiInsensitive,
                ) if starts_with_ignore_ascii_case(element_value, value) => {}
                (
                    SelectorAttributeOperator::Suffix,
                    Some(value),
                    SelectorAttributeCaseSensitivity::CaseSensitive,
                ) if element_value.ends_with(value) => {}
                (
                    SelectorAttributeOperator::Suffix,
                    Some(value),
                    SelectorAttributeCaseSensitivity::AsciiInsensitive,
                ) if ends_with_ignore_ascii_case(element_value, value) => {}
                (
                    SelectorAttributeOperator::Contains,
                    Some(value),
                    SelectorAttributeCaseSensitivity::CaseSensitive,
                ) if element_value.contains(value) => {}
                (
                    SelectorAttributeOperator::Contains,
                    Some(value),
                    SelectorAttributeCaseSensitivity::AsciiInsensitive,
                ) if contains_ignore_ascii_case(element_value, value) => {}
                (
                    SelectorAttributeOperator::Includes,
                    Some(value),
                    SelectorAttributeCaseSensitivity::CaseSensitive,
                ) if element_value
                    .split_ascii_whitespace()
                    .any(|candidate| candidate == value) => {}
                (
                    SelectorAttributeOperator::Includes,
                    Some(value),
                    SelectorAttributeCaseSensitivity::AsciiInsensitive,
                ) if element_value
                    .split_ascii_whitespace()
                    .any(|candidate| candidate.eq_ignore_ascii_case(value)) => {}
                (
                    SelectorAttributeOperator::DashMatch,
                    Some(value),
                    SelectorAttributeCaseSensitivity::CaseSensitive,
                ) if element_value == value
                    || (!value.is_empty()
                        && element_value
                            .strip_prefix(value)
                            .is_some_and(|rest| rest.starts_with('-'))) => {}
                (
                    SelectorAttributeOperator::DashMatch,
                    Some(value),
                    SelectorAttributeCaseSensitivity::AsciiInsensitive,
                ) if element_value.eq_ignore_ascii_case(value)
                    || (!value.is_empty()
                        && starts_with_ignore_ascii_case(element_value, value)
                        && element_value
                            .get(value.len()..)
                            .is_some_and(|rest| rest.starts_with('-'))) => {}
                _ => {
                    return false;
                }
            }
        }

        for pseudo_class in &query.pseudo_classes {
            if !self.matches_selector_pseudo_class(node_id, pseudo_class) {
                return false;
            }
        }

        true
    }

    fn matches_selector_pseudo_class(
        &self,
        node_id: NodeId,
        pseudo_class: &SelectorPseudoClass,
    ) -> bool {
        match pseudo_class {
            SelectorPseudoClass::Root => self.is_root_pseudo_class(node_id),
            SelectorPseudoClass::Empty => self.is_empty_pseudo_class(node_id),
            SelectorPseudoClass::OnlyChild => self.is_only_child_pseudo_class(node_id),
            SelectorPseudoClass::OnlyOfType => self.is_only_of_type_pseudo_class(node_id),
            SelectorPseudoClass::FirstChild => self.is_first_child(node_id),
            SelectorPseudoClass::LastChild => self.is_last_child(node_id),
            SelectorPseudoClass::FirstOfType => self.is_first_of_type(node_id),
            SelectorPseudoClass::LastOfType => self.is_last_of_type(node_id),
            SelectorPseudoClass::NthChild(pattern) => self.is_nth_child(node_id, pattern),
            SelectorPseudoClass::NthLastChild(pattern) => self.is_nth_last_child(node_id, pattern),
            SelectorPseudoClass::NthOfType(pattern) => self.is_nth_of_type(node_id, pattern),
            SelectorPseudoClass::NthLastOfType(pattern) => {
                self.is_nth_last_of_type(node_id, pattern)
            }
            SelectorPseudoClass::Is(chains) => chains
                .iter()
                .any(|chain| self.matches_selector_chain(node_id, chain)),
            SelectorPseudoClass::Where(chains) => chains
                .iter()
                .any(|chain| self.matches_selector_chain(node_id, chain)),
            SelectorPseudoClass::Not(chains) => !chains
                .iter()
                .any(|chain| self.matches_selector_chain(node_id, chain)),
            SelectorPseudoClass::Checked => self.is_checked_pseudo_class(node_id),
            SelectorPseudoClass::Disabled => self.is_disabled_pseudo_class(node_id),
            SelectorPseudoClass::Enabled => self.is_enabled_pseudo_class(node_id),
        }
    }

    fn parse_selector_chain(selector: &str) -> Result<SelectorChain, String> {
        let mut parts = Vec::new();
        let mut relations = Vec::new();
        let bytes = selector.as_bytes();
        let mut pos = 0;

        parts.push(Self::parse_selector_compound(selector, &mut pos)?);

        while pos < bytes.len() {
            let had_whitespace = skip_selector_whitespace(bytes, &mut pos);
            if pos >= bytes.len() {
                break;
            }

            let relation = match bytes[pos] {
                b'>' => {
                    pos += 1;
                    SelectorCombinator::Child
                }
                b'+' => {
                    pos += 1;
                    SelectorCombinator::AdjacentSibling
                }
                b'~' => {
                    pos += 1;
                    SelectorCombinator::GeneralSibling
                }
                byte if is_selector_combinator_byte(byte) => {
                    return Err(selector_not_supported(selector));
                }
                _ if had_whitespace => SelectorCombinator::Descendant,
                _ => return Err(selector_not_supported(selector)),
            };

            skip_selector_whitespace(bytes, &mut pos);
            if pos >= bytes.len() {
                return Err(selector_not_supported(selector));
            }

            let part = Self::parse_selector_compound(selector, &mut pos)?;
            relations.push(relation);
            parts.push(part);
        }

        if parts.is_empty() {
            return Err(selector_not_supported(selector));
        }

        Ok(SelectorChain { parts, relations })
    }

    fn parse_selector_list(selector: &str) -> Result<Vec<SelectorChain>, String> {
        let mut chains = Vec::new();

        for item in split_selector_list_items(selector)? {
            chains.push(Self::parse_selector_chain(item)?);
        }

        if chains.is_empty() {
            return Err(selector_not_supported(selector));
        }

        Ok(chains)
    }

    fn parse_selector_compound(selector: &str, pos: &mut usize) -> Result<SelectorQuery, String> {
        let mut query = SelectorQuery::default();
        let bytes = selector.as_bytes();
        let mut saw_token = false;

        while *pos < bytes.len() {
            if bytes[*pos].is_ascii_whitespace() || is_selector_combinator_byte(bytes[*pos]) {
                break;
            }

            match bytes[*pos] {
                b'#' => {
                    *pos += 1;
                    let token = parse_selector_token(selector, pos)?;
                    if query.id.is_some() {
                        return Err(selector_not_supported(selector));
                    }
                    query.id = Some(token);
                    saw_token = true;
                }
                b'.' => {
                    *pos += 1;
                    let token = parse_selector_token(selector, pos)?;
                    query.classes.push(token);
                    saw_token = true;
                }
                b'[' => {
                    *pos += 1;
                    skip_selector_whitespace(bytes, pos);
                    let name = parse_selector_token(selector, pos)?.to_ascii_lowercase();
                    skip_selector_whitespace(bytes, pos);

                    let (operator, value, case_sensitivity) =
                        if *pos < bytes.len() && bytes[*pos] != b']' {
                            parse_selector_attribute_operator_and_value(selector, pos)?
                        } else {
                            (
                                SelectorAttributeOperator::Exists,
                                None,
                                SelectorAttributeCaseSensitivity::CaseSensitive,
                            )
                        };

                    skip_selector_whitespace(bytes, pos);
                    if *pos >= bytes.len() || bytes[*pos] != b']' {
                        return Err(selector_not_supported(selector));
                    }
                    *pos += 1;
                    query.attributes.push(SelectorAttribute {
                        name,
                        operator,
                        value,
                        case_sensitivity,
                    });
                    saw_token = true;
                }
                b':' => {
                    *pos += 1;
                    let token = parse_selector_token(selector, pos)?;
                    let pseudo_class = match token.as_str() {
                        "root" => SelectorPseudoClass::Root,
                        "empty" => SelectorPseudoClass::Empty,
                        "only-child" => SelectorPseudoClass::OnlyChild,
                        "only-of-type" => SelectorPseudoClass::OnlyOfType,
                        "first-child" => SelectorPseudoClass::FirstChild,
                        "last-child" => SelectorPseudoClass::LastChild,
                        "first-of-type" => SelectorPseudoClass::FirstOfType,
                        "last-of-type" => SelectorPseudoClass::LastOfType,
                        "nth-child" => {
                            SelectorPseudoClass::NthChild(parse_nth_child_argument(selector, pos)?)
                        }
                        "nth-last-child" => SelectorPseudoClass::NthLastChild(
                            parse_nth_child_argument(selector, pos)?,
                        ),
                        "nth-of-type" => {
                            SelectorPseudoClass::NthOfType(parse_nth_child_argument(selector, pos)?)
                        }
                        "nth-last-of-type" => SelectorPseudoClass::NthLastOfType(
                            parse_nth_child_argument(selector, pos)?,
                        ),
                        "is" => {
                            SelectorPseudoClass::Is(parse_logical_pseudo_argument(selector, pos)?)
                        }
                        "where" => SelectorPseudoClass::Where(parse_logical_pseudo_argument(
                            selector, pos,
                        )?),
                        "not" => {
                            SelectorPseudoClass::Not(parse_logical_pseudo_argument(selector, pos)?)
                        }
                        "checked" => SelectorPseudoClass::Checked,
                        "disabled" => SelectorPseudoClass::Disabled,
                        "enabled" => SelectorPseudoClass::Enabled,
                        _ => return Err(selector_not_supported(selector)),
                    };
                    query.pseudo_classes.push(pseudo_class);
                    saw_token = true;
                }
                byte if is_simple_name_byte(byte) => {
                    let token = parse_selector_token(selector, pos)?;
                    if query.tag.is_some() {
                        return Err(selector_not_supported(selector));
                    }
                    query.tag = Some(token.to_ascii_lowercase());
                    saw_token = true;
                }
                _ => return Err(selector_not_supported(selector)),
            }
        }

        if !saw_token {
            return Err(selector_not_supported(selector));
        }

        Ok(query)
    }

    fn parent_of(&self, node_id: NodeId) -> Option<NodeId> {
        self.nodes
            .get(node_id.index() as usize)
            .and_then(|node| node.parent)
    }

    fn root_element_id(&self) -> Option<NodeId> {
        let document = self.nodes.get(self.document_id.index() as usize)?;
        document.children.iter().find_map(|child| {
            matches!(
                self.nodes
                    .get(child.index() as usize)
                    .map(|node| &node.kind),
                Some(NodeKind::Element(_))
            )
            .then_some(*child)
        })
    }

    fn is_root_pseudo_class(&self, node_id: NodeId) -> bool {
        self.root_element_id() == Some(node_id)
    }

    fn is_empty_pseudo_class(&self, node_id: NodeId) -> bool {
        let Some(node) = self.nodes.get(node_id.index() as usize) else {
            return false;
        };
        let NodeKind::Element(_) = node.kind else {
            return false;
        };

        !node.children.iter().any(|child| {
            matches!(
                self.nodes
                    .get(child.index() as usize)
                    .map(|child_node| &child_node.kind),
                Some(NodeKind::Element(_)) | Some(NodeKind::Text(_))
            )
        })
    }

    fn is_only_child_pseudo_class(&self, node_id: NodeId) -> bool {
        self.is_first_child(node_id) && self.is_last_child(node_id)
    }

    fn is_only_of_type_pseudo_class(&self, node_id: NodeId) -> bool {
        self.element_sibling_position_of_type(node_id) == Some(1)
            && self.element_sibling_position_from_end_of_type(node_id) == Some(1)
    }

    fn is_first_of_type(&self, node_id: NodeId) -> bool {
        self.element_sibling_position_of_type(node_id) == Some(1)
    }

    fn is_last_of_type(&self, node_id: NodeId) -> bool {
        self.element_sibling_position_from_end_of_type(node_id) == Some(1)
    }

    fn is_nth_of_type(&self, node_id: NodeId, pattern: &SelectorNthChildPattern) -> bool {
        let Some(position) = self.element_sibling_position_of_type(node_id) else {
            return false;
        };

        self.matches_nth_pattern(position as isize, pattern)
    }

    fn is_nth_last_of_type(&self, node_id: NodeId, pattern: &SelectorNthChildPattern) -> bool {
        let Some(position) = self.element_sibling_position_from_end_of_type(node_id) else {
            return false;
        };

        self.matches_nth_pattern(position as isize, pattern)
    }

    fn element_sibling_position_of_type(&self, node_id: NodeId) -> Option<usize> {
        let Some(node) = self.nodes.get(node_id.index() as usize) else {
            return None;
        };
        let NodeKind::Element(element) = &node.kind else {
            return None;
        };

        let Some(parent_id) = node.parent else {
            return None;
        };
        let Some(parent) = self.nodes.get(parent_id.index() as usize) else {
            return None;
        };

        let mut matching_sibling_count = 0usize;
        for child in &parent.children {
            let Some(child_node) = self.nodes.get(child.index() as usize) else {
                continue;
            };
            let NodeKind::Element(child_element) = &child_node.kind else {
                continue;
            };

            if child_element.local_name == element.local_name
                && child_element.namespace_uri == element.namespace_uri
            {
                matching_sibling_count += 1;
                if *child == node_id {
                    return Some(matching_sibling_count);
                }
            } else if *child == node_id {
                return None;
            }
        }

        None
    }

    fn element_sibling_position_from_end_of_type(&self, node_id: NodeId) -> Option<usize> {
        let Some(node) = self.nodes.get(node_id.index() as usize) else {
            return None;
        };
        let NodeKind::Element(element) = &node.kind else {
            return None;
        };

        let Some(parent_id) = node.parent else {
            return None;
        };
        let Some(parent) = self.nodes.get(parent_id.index() as usize) else {
            return None;
        };

        let mut matching_sibling_count = 0usize;
        for child in parent.children.iter().rev() {
            let Some(child_node) = self.nodes.get(child.index() as usize) else {
                continue;
            };
            let NodeKind::Element(child_element) = &child_node.kind else {
                continue;
            };

            if child_element.local_name == element.local_name
                && child_element.namespace_uri == element.namespace_uri
            {
                matching_sibling_count += 1;
                if *child == node_id {
                    return Some(matching_sibling_count);
                }
            } else if *child == node_id {
                return None;
            }
        }

        None
    }

    fn is_first_child(&self, node_id: NodeId) -> bool {
        self.element_child_position(node_id) == Some(1)
    }

    fn is_last_child(&self, node_id: NodeId) -> bool {
        let Some(parent_id) = self.parent_of(node_id) else {
            return false;
        };
        let Some(parent) = self.nodes.get(parent_id.index() as usize) else {
            return false;
        };

        parent.children.iter().rev().find_map(|child| {
            matches!(
                self.nodes
                    .get(child.index() as usize)
                    .map(|node| &node.kind),
                Some(NodeKind::Element(_))
            )
            .then_some(*child)
        }) == Some(node_id)
    }

    fn is_nth_child(&self, node_id: NodeId, pattern: &SelectorNthChildPattern) -> bool {
        let Some(position) = self.element_child_position(node_id) else {
            return false;
        };

        self.matches_nth_pattern(position as isize, pattern)
    }

    fn is_nth_last_child(&self, node_id: NodeId, pattern: &SelectorNthChildPattern) -> bool {
        let Some(position) = self.element_child_position_from_end(node_id) else {
            return false;
        };

        self.matches_nth_pattern(position as isize, pattern)
    }

    fn matches_nth_pattern(&self, position: isize, pattern: &SelectorNthChildPattern) -> bool {
        match pattern.step.cmp(&0) {
            std::cmp::Ordering::Equal => position == pattern.offset && position > 0,
            std::cmp::Ordering::Greater => {
                let diff = position - pattern.offset;
                diff >= 0 && diff % pattern.step == 0
            }
            std::cmp::Ordering::Less => {
                let step = -pattern.step;
                let diff = pattern.offset - position;
                diff >= 0 && diff % step == 0
            }
        }
    }

    fn is_checked_pseudo_class(&self, node_id: NodeId) -> bool {
        let Some(node) = self.nodes.get(node_id.index() as usize) else {
            return false;
        };

        let NodeKind::Element(element) = &node.kind else {
            return false;
        };

        if element.tag_name == "option" {
            return self.is_option_selected(node_id);
        }

        self.checked_for_node(node_id) == Some(true)
    }

    fn is_disabled_pseudo_class(&self, node_id: NodeId) -> bool {
        let Some(node) = self.nodes.get(node_id.index() as usize) else {
            return false;
        };

        let NodeKind::Element(element) = &node.kind else {
            return false;
        };

        if !supports_disabled_pseudo_class(&element.tag_name) {
            return false;
        }

        element.attributes.contains_key("disabled")
    }

    fn is_enabled_pseudo_class(&self, node_id: NodeId) -> bool {
        let Some(node) = self.nodes.get(node_id.index() as usize) else {
            return false;
        };

        let NodeKind::Element(element) = &node.kind else {
            return false;
        };

        if !supports_disabled_pseudo_class(&element.tag_name) {
            return false;
        }

        !element.attributes.contains_key("disabled")
    }

    fn previous_element_sibling_of(&self, node_id: NodeId) -> Option<NodeId> {
        let parent_id = self.parent_of(node_id)?;
        let parent = self.nodes.get(parent_id.index() as usize)?;
        let mut previous_element = None;

        for child in &parent.children {
            if *child == node_id {
                return previous_element;
            }

            if matches!(
                self.nodes
                    .get(child.index() as usize)
                    .map(|node| &node.kind),
                Some(NodeKind::Element(_))
            ) {
                previous_element = Some(*child);
            }
        }

        None
    }

    fn element_child_position(&self, node_id: NodeId) -> Option<usize> {
        let parent_id = self.parent_of(node_id)?;
        let parent = self.nodes.get(parent_id.index() as usize)?;
        let mut position = 0;

        for child in &parent.children {
            if matches!(
                self.nodes
                    .get(child.index() as usize)
                    .map(|node| &node.kind),
                Some(NodeKind::Element(_))
            ) {
                position += 1;
                if *child == node_id {
                    return Some(position);
                }
            } else if *child == node_id {
                return None;
            }
        }

        None
    }

    fn element_child_position_from_end(&self, node_id: NodeId) -> Option<usize> {
        let parent_id = self.parent_of(node_id)?;
        let parent = self.nodes.get(parent_id.index() as usize)?;
        let mut position = 0;

        for child in parent.children.iter().rev() {
            if matches!(
                self.nodes
                    .get(child.index() as usize)
                    .map(|node| &node.kind),
                Some(NodeKind::Element(_))
            ) {
                position += 1;
                if *child == node_id {
                    return Some(position);
                }
            } else if *child == node_id {
                return None;
            }
        }

        None
    }

    fn dump_node(&self, node_id: NodeId, indent: usize, output: &mut String) {
        let node = &self.nodes[node_id.index() as usize];
        let children = node.children.clone();

        match &node.kind {
            NodeKind::Document => {
                write_indent(output, indent);
                output.push_str("#document");
                if !children.is_empty() {
                    output.push('\n');
                    for (index, child) in children.iter().enumerate() {
                        self.dump_node(*child, indent + 1, output);
                        if index + 1 < children.len() {
                            output.push('\n');
                        }
                    }
                }
            }
            NodeKind::Element(element) => {
                let attributes = format_attributes(&element.attributes);
                write_indent(output, indent);
                if children.is_empty() {
                    if attributes.is_empty() {
                        let _ = write!(output, "<{} />", element.tag_name);
                    } else {
                        let _ = write!(output, "<{} {} />", element.tag_name, attributes);
                    }
                } else {
                    if attributes.is_empty() {
                        let _ = write!(output, "<{}>", element.tag_name);
                    } else {
                        let _ = write!(output, "<{} {}>", element.tag_name, attributes);
                    }
                    output.push('\n');
                    for (index, child) in children.iter().enumerate() {
                        self.dump_node(*child, indent + 1, output);
                        if index + 1 < children.len() {
                            output.push('\n');
                        }
                    }
                    output.push('\n');
                    write_indent(output, indent);
                    let _ = write!(output, "</{}>", element.tag_name);
                }
            }
            NodeKind::Text(text) => {
                write_indent(output, indent);
                let _ = write!(output, "\"{}\"", escape_text(&text.value));
            }
            NodeKind::Comment(comment) => {
                write_indent(output, indent);
                let _ = write!(output, "<!-- {} -->", comment);
            }
        }
    }

    fn tag_name_for(&self, node_id: NodeId) -> Option<&str> {
        match &self.nodes[node_id.index() as usize].kind {
            NodeKind::Element(element) => Some(element.tag_name.as_str()),
            _ => None,
        }
    }
}

fn selector_not_supported(selector: &str) -> String {
    format!(
        "unsupported selector `{selector}`; supported forms are #id, .class, tag, tag.class, #id.class, [attr], [attr=value], [attr^=value], [attr$=value], [attr*=value], [attr~=value], [attr|=value], optional attribute selector flags like `[attr=value i]` and `[attr=value s]`, bounded logical pseudo-classes like `:not(.primary)`, `:is(.primary, .secondary)`, and `:where(.primary, .secondary)`, structural pseudo-classes like `:first-child`, `:last-child`, `:nth-child(2)`, `:nth-child(odd)`, `:nth-child(2n+1)`, and `:nth-last-child(2)`, state pseudo-classes like `:checked`, `:disabled`, and `:enabled`, descendant combinators like `A B`, adjacent sibling combinators like `A + B`, general sibling combinators like `A ~ B`, and child combinators like `A > B`; additional bounded structural pseudo-classes include `:root`, `:empty`, `:only-child`, `:only-of-type`, `:first-of-type`, `:last-of-type`, `:nth-of-type(2)`, and `:nth-last-of-type(2)`"
    )
}

fn parse_nth_child_argument(
    selector: &str,
    pos: &mut usize,
) -> Result<SelectorNthChildPattern, String> {
    let mut formula: String = parse_parenthesized_argument(selector, pos)?
        .chars()
        .filter(|ch| !ch.is_ascii_whitespace())
        .collect();

    if formula.is_empty() {
        return Err(selector_not_supported(selector));
    }

    formula.make_ascii_lowercase();
    match formula.as_str() {
        "odd" => {
            return Ok(SelectorNthChildPattern { step: 2, offset: 1 });
        }
        "even" => {
            return Ok(SelectorNthChildPattern { step: 2, offset: 0 });
        }
        _ => {}
    }

    if let Some(n_index) = formula.find('n') {
        if formula[n_index + 1..].contains('n') {
            return Err(selector_not_supported(selector));
        }

        let step = match &formula[..n_index] {
            "" | "+" => 1,
            "-" => -1,
            value => value
                .parse::<isize>()
                .map_err(|_| selector_not_supported(selector))?,
        };
        let offset = if formula[n_index + 1..].is_empty() {
            0
        } else {
            formula[n_index + 1..]
                .parse::<isize>()
                .map_err(|_| selector_not_supported(selector))?
        };

        return Ok(SelectorNthChildPattern { step, offset });
    }

    let offset = formula
        .parse::<isize>()
        .map_err(|_| selector_not_supported(selector))?;
    Ok(SelectorNthChildPattern { step: 0, offset })
}

fn parse_logical_pseudo_argument(
    selector: &str,
    pos: &mut usize,
) -> Result<Vec<SelectorChain>, String> {
    let argument = parse_parenthesized_argument(selector, pos)?;
    let mut chains = Vec::new();
    for item in
        split_selector_list_items(&argument).map_err(|_| selector_not_supported(selector))?
    {
        chains.push(
            DomStore::parse_selector_chain(item).map_err(|_| selector_not_supported(selector))?,
        );
    }

    if chains.is_empty() {
        return Err(selector_not_supported(selector));
    }

    Ok(chains)
}

fn parse_selector_attribute_operator_and_value(
    selector: &str,
    pos: &mut usize,
) -> Result<
    (
        SelectorAttributeOperator,
        Option<String>,
        SelectorAttributeCaseSensitivity,
    ),
    String,
> {
    let bytes = selector.as_bytes();
    let Some(current) = bytes.get(*pos).copied() else {
        return Err(selector_not_supported(selector));
    };

    let operator = match current {
        b'=' => {
            *pos += 1;
            SelectorAttributeOperator::Exact
        }
        b'^' => {
            if bytes.get(*pos + 1) != Some(&b'=') {
                return Err(selector_not_supported(selector));
            }
            *pos += 2;
            SelectorAttributeOperator::Prefix
        }
        b'$' => {
            if bytes.get(*pos + 1) != Some(&b'=') {
                return Err(selector_not_supported(selector));
            }
            *pos += 2;
            SelectorAttributeOperator::Suffix
        }
        b'*' => {
            if bytes.get(*pos + 1) != Some(&b'=') {
                return Err(selector_not_supported(selector));
            }
            *pos += 2;
            SelectorAttributeOperator::Contains
        }
        b'~' => {
            if bytes.get(*pos + 1) != Some(&b'=') {
                return Err(selector_not_supported(selector));
            }
            *pos += 2;
            SelectorAttributeOperator::Includes
        }
        b'|' => {
            if bytes.get(*pos + 1) != Some(&b'=') {
                return Err(selector_not_supported(selector));
            }
            *pos += 2;
            SelectorAttributeOperator::DashMatch
        }
        _ => return Err(selector_not_supported(selector)),
    };

    skip_selector_whitespace(bytes, pos);
    let value = parse_selector_attribute_value(selector, pos)?;
    skip_selector_whitespace(bytes, pos);
    let case_sensitivity = parse_selector_attribute_case_sensitivity(selector, pos)?;
    Ok((operator, Some(value), case_sensitivity))
}

fn parse_selector_attribute_value(selector: &str, pos: &mut usize) -> Result<String, String> {
    let bytes = selector.as_bytes();
    match bytes.get(*pos).copied() {
        Some(quote @ (b'"' | b'\'')) => {
            *pos += 1;
            let mut value = String::new();
            while *pos < bytes.len() {
                match bytes[*pos] {
                    b'\\' => {
                        *pos += 1;
                        value.push(skip_selector_escape(selector, pos)?);
                    }
                    byte if byte == quote => {
                        *pos += 1;
                        return Ok(value);
                    }
                    _ => {
                        let ch = selector[*pos..]
                            .chars()
                            .next()
                            .ok_or_else(|| selector_not_supported(selector))?;
                        value.push(ch);
                        *pos += ch.len_utf8();
                    }
                }
            }

            Err(selector_not_supported(selector))
        }
        Some(_) => {
            let mut value = String::new();
            while *pos < bytes.len() {
                match bytes[*pos] {
                    b'\\' => {
                        *pos += 1;
                        value.push(skip_selector_escape(selector, pos)?);
                    }
                    byte if byte.is_ascii_whitespace() || byte == b']' => break,
                    _ => {
                        let ch = selector[*pos..]
                            .chars()
                            .next()
                            .ok_or_else(|| selector_not_supported(selector))?;
                        value.push(ch);
                        *pos += ch.len_utf8();
                    }
                }
            }

            if value.is_empty() {
                return Err(selector_not_supported(selector));
            }

            Ok(value)
        }
        None => Err(selector_not_supported(selector)),
    }
}

fn parse_selector_attribute_case_sensitivity(
    selector: &str,
    pos: &mut usize,
) -> Result<SelectorAttributeCaseSensitivity, String> {
    let bytes = selector.as_bytes();
    match bytes.get(*pos).copied() {
        Some(b']') | None => Ok(SelectorAttributeCaseSensitivity::CaseSensitive),
        Some(flag) => {
            *pos += 1;
            let case_sensitivity = match flag.to_ascii_lowercase() {
                b'i' => SelectorAttributeCaseSensitivity::AsciiInsensitive,
                b's' => SelectorAttributeCaseSensitivity::CaseSensitive,
                _ => return Err(selector_not_supported(selector)),
            };
            skip_selector_whitespace(bytes, pos);
            if bytes.get(*pos) != Some(&b']') {
                return Err(selector_not_supported(selector));
            }
            Ok(case_sensitivity)
        }
    }
}

fn starts_with_ignore_ascii_case(value: &str, prefix: &str) -> bool {
    value
        .get(..prefix.len())
        .is_some_and(|candidate| candidate.eq_ignore_ascii_case(prefix))
}

fn ends_with_ignore_ascii_case(value: &str, suffix: &str) -> bool {
    value
        .get(value.len().saturating_sub(suffix.len())..)
        .is_some_and(|candidate| candidate.eq_ignore_ascii_case(suffix))
}

fn contains_ignore_ascii_case(value: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }

    value
        .to_ascii_lowercase()
        .contains(&needle.to_ascii_lowercase())
}

fn parse_parenthesized_argument(selector: &str, pos: &mut usize) -> Result<String, String> {
    let bytes = selector.as_bytes();
    if *pos >= bytes.len() || bytes[*pos] != b'(' {
        return Err(selector_not_supported(selector));
    }
    *pos += 1;

    let start = *pos;
    let mut depth = 1usize;
    let mut in_quote: Option<u8> = None;
    let mut bracket_depth = 0usize;
    while *pos < bytes.len() {
        let byte = bytes[*pos];
        match in_quote {
            Some(quote) => match byte {
                b'\\' => {
                    *pos += 1;
                    skip_selector_escape(selector, pos)?;
                    continue;
                }
                byte if byte == quote => {
                    in_quote = None;
                    *pos += 1;
                    continue;
                }
                _ => {
                    let ch = selector[*pos..]
                        .chars()
                        .next()
                        .ok_or_else(|| selector_not_supported(selector))?;
                    *pos += ch.len_utf8();
                    continue;
                }
            },
            None => match byte {
                b'\'' | b'"' => {
                    in_quote = Some(byte);
                    *pos += 1;
                }
                b'\\' => {
                    *pos += 1;
                    skip_selector_escape(selector, pos)?;
                    continue;
                }
                b'[' => {
                    bracket_depth += 1;
                    *pos += 1;
                }
                b']' => {
                    if bracket_depth == 0 {
                        return Err(selector_not_supported(selector));
                    }
                    bracket_depth -= 1;
                    *pos += 1;
                }
                b'(' if bracket_depth == 0 => {
                    depth += 1;
                    *pos += 1;
                }
                b')' if bracket_depth == 0 => {
                    depth -= 1;
                    if depth == 0 {
                        let argument = selector[start..*pos].to_string();
                        *pos += 1;
                        return Ok(argument);
                    }
                    *pos += 1;
                }
                _ => {
                    let ch = selector[*pos..]
                        .chars()
                        .next()
                        .ok_or_else(|| selector_not_supported(selector))?;
                    *pos += ch.len_utf8();
                }
            },
        }
    }

    Err(selector_not_supported(selector))
}

fn split_selector_list_items(selector: &str) -> Result<Vec<&str>, String> {
    let bytes = selector.as_bytes();
    let mut items = Vec::new();
    let mut depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut in_quote: Option<u8> = None;
    let mut start = 0usize;
    let mut pos = 0usize;
    while pos < bytes.len() {
        let byte = bytes[pos];
        match in_quote {
            Some(quote) => match byte {
                b'\\' => {
                    pos += 1;
                    skip_selector_escape(selector, &mut pos)?;
                    continue;
                }
                byte if byte == quote => {
                    in_quote = None;
                    pos += 1;
                    continue;
                }
                _ => {
                    let ch = selector[pos..]
                        .chars()
                        .next()
                        .ok_or_else(|| selector_not_supported(selector))?;
                    pos += ch.len_utf8();
                    continue;
                }
            },
            None => match byte {
                b'\'' | b'"' => {
                    in_quote = Some(byte);
                    pos += 1;
                }
                b'\\' => {
                    pos += 1;
                    skip_selector_escape(selector, &mut pos)?;
                }
                b'(' => {
                    depth += 1;
                    pos += 1;
                }
                b')' => {
                    if depth == 0 {
                        return Err(selector_not_supported(selector));
                    }
                    depth -= 1;
                    pos += 1;
                }
                b'[' => {
                    bracket_depth += 1;
                    pos += 1;
                }
                b']' => {
                    if bracket_depth == 0 {
                        return Err(selector_not_supported(selector));
                    }
                    bracket_depth -= 1;
                    pos += 1;
                }
                b',' if depth == 0 && bracket_depth == 0 => {
                    let item = selector[start..pos].trim();
                    if item.is_empty() {
                        return Err(selector_not_supported(selector));
                    }
                    items.push(item);
                    pos += 1;
                    start = pos;
                }
                _ => {
                    let ch = selector[pos..]
                        .chars()
                        .next()
                        .ok_or_else(|| selector_not_supported(selector))?;
                    pos += ch.len_utf8();
                }
            },
        }
    }

    if depth != 0 || bracket_depth != 0 || in_quote.is_some() {
        return Err(selector_not_supported(selector));
    }

    let item = selector[start..].trim();
    if item.is_empty() {
        return Err(selector_not_supported(selector));
    }
    items.push(item);

    Ok(items)
}

fn parse_selector_token(selector: &str, pos: &mut usize) -> Result<String, String> {
    let bytes = selector.as_bytes();
    let mut token = String::new();

    while *pos < bytes.len() {
        let byte = bytes[*pos];
        if byte == b'\\' {
            *pos += 1;
            token.push(skip_selector_escape(selector, pos)?);
            continue;
        }

        if is_simple_name_byte(byte) {
            token.push(byte as char);
            *pos += 1;
            continue;
        }

        break;
    }

    if token.is_empty() {
        return Err(selector_not_supported(selector));
    }

    Ok(token)
}

fn skip_selector_escape(selector: &str, pos: &mut usize) -> Result<char, String> {
    if *pos >= selector.len() {
        return Err(selector_not_supported(selector));
    }

    let bytes = selector.as_bytes();
    let mut end = *pos;
    let mut digits = 0usize;
    while end < bytes.len() && digits < 6 && bytes[end].is_ascii_hexdigit() {
        end += 1;
        digits += 1;
    }

    if digits > 0 {
        let value = u32::from_str_radix(&selector[*pos..end], 16)
            .map_err(|_| selector_not_supported(selector))?;
        let ch = char::from_u32(value).ok_or_else(|| selector_not_supported(selector))?;
        if ch.is_control() {
            return Err(selector_not_supported(selector));
        }
        *pos = end;

        if *pos < bytes.len() && bytes[*pos].is_ascii_whitespace() {
            *pos += 1;
        }

        return Ok(ch);
    }

    let ch = selector[*pos..]
        .chars()
        .next()
        .ok_or_else(|| selector_not_supported(selector))?;
    *pos += ch.len_utf8();
    Ok(ch)
}

fn skip_selector_whitespace(bytes: &[u8], pos: &mut usize) -> bool {
    let start = *pos;
    while *pos < bytes.len() && bytes[*pos].is_ascii_whitespace() {
        *pos += 1;
    }

    *pos != start
}

fn is_selector_combinator_byte(byte: u8) -> bool {
    matches!(byte, b'>' | b'+' | b'~' | b',')
}

struct HtmlParser<'a> {
    input: &'a str,
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> HtmlParser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            bytes: input.as_bytes(),
            pos: 0,
        }
    }

    fn parse_into(&mut self, store: &mut DomStore) -> Result<(), String> {
        let mut stack = vec![store.document_id];
        while self.pos < self.bytes.len() {
            if self.bytes[self.pos] == b'<' {
                if self.starts_with_bytes(b"<!--") {
                    let parent = *stack
                        .last()
                        .expect("document root should always be on stack");
                    self.parse_comment(store, parent)?;
                    continue;
                }

                if self.starts_with_bytes(b"</") {
                    self.parse_closing_tag(store, &mut stack)?;
                    continue;
                }

                if self.starts_with_bytes(b"<!") {
                    self.parse_declaration()?;
                    continue;
                }

                self.parse_start_tag(store, &mut stack)?;
                continue;
            }

            let parent = *stack
                .last()
                .expect("document root should always be on stack");
            self.parse_text(store, parent)?;
        }

        if stack.len() != 1 {
            let open_id = *stack
                .last()
                .expect("document root should always be on stack");
            let tag_name = store.tag_name_for(open_id).unwrap_or("unknown").to_string();
            return Err(format!("unclosed tag <{}>", tag_name));
        }

        Ok(())
    }

    fn starts_with_bytes(&self, pattern: &[u8]) -> bool {
        self.bytes[self.pos..].starts_with(pattern)
    }

    fn current_byte(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn skip_ascii_whitespace(&mut self) {
        while matches!(
            self.current_byte(),
            Some(b' ' | b'\n' | b'\r' | b'\t' | 0x0c)
        ) {
            self.pos += 1;
        }
    }

    fn parse_text(&mut self, store: &mut DomStore, parent: NodeId) -> Result<(), String> {
        let rest = &self.input[self.pos..];
        let next_tag = rest.find('<').unwrap_or(rest.len());
        let value = &rest[..next_tag];
        self.pos += next_tag;

        if !value.is_empty() {
            store.add_text(parent, value.to_string());
        }

        Ok(())
    }

    fn parse_comment(&mut self, store: &mut DomStore, parent: NodeId) -> Result<(), String> {
        self.pos += 4;
        let rest = &self.input[self.pos..];
        let end = rest
            .find("-->")
            .ok_or_else(|| format!("unterminated comment at byte {}", self.pos - 4))?;
        let value = &rest[..end];
        self.pos += end + 3;
        store.add_comment(parent, value.to_string());
        Ok(())
    }

    fn parse_declaration(&mut self) -> Result<(), String> {
        self.pos += 2;
        let rest = &self.input[self.pos..];
        let end = rest
            .find('>')
            .ok_or_else(|| format!("unterminated declaration at byte {}", self.pos - 2))?;
        self.pos += end + 1;
        Ok(())
    }

    fn parse_start_tag(
        &mut self,
        store: &mut DomStore,
        stack: &mut Vec<NodeId>,
    ) -> Result<(), String> {
        self.pos += 1;
        if self.pos >= self.bytes.len() {
            return Err("unexpected end of input after `<`".to_string());
        }

        if !self
            .current_byte()
            .map(is_simple_name_byte)
            .unwrap_or(false)
        {
            return Err(format!("invalid tag name at byte {}", self.pos));
        }

        let tag_name = self.parse_name_token("tag")?;
        let mut attributes = BTreeMap::new();
        let start_tag_name = tag_name.clone();

        loop {
            self.skip_ascii_whitespace();
            if self.pos >= self.bytes.len() {
                return Err(format!("unclosed start tag <{}>", start_tag_name));
            }

            if self.starts_with_bytes(b"/>") {
                self.pos += 2;
                self.finish_start_tag(store, stack, tag_name, attributes, true);
                return Ok(());
            }

            if self.current_byte() == Some(b'>') {
                self.pos += 1;
                self.finish_start_tag(store, stack, tag_name, attributes, false);
                return Ok(());
            }

            let attribute_name = self.parse_name_token("attribute")?;
            self.skip_ascii_whitespace();

            let value = if self.current_byte() == Some(b'=') {
                self.pos += 1;
                self.skip_ascii_whitespace();
                self.parse_attribute_value()?
            } else {
                String::new()
            };

            attributes.insert(attribute_name, value);
        }
    }

    fn finish_start_tag(
        &mut self,
        store: &mut DomStore,
        stack: &mut Vec<NodeId>,
        tag_name: String,
        attributes: BTreeMap<String, String>,
        self_closing: bool,
    ) {
        let parent = *stack
            .last()
            .expect("document root should always be on stack");
        let node_id = store.add_element(parent, tag_name.clone(), attributes);
        if !self_closing && !is_void_element(&tag_name) {
            stack.push(node_id);
        }
    }

    fn parse_closing_tag(
        &mut self,
        store: &mut DomStore,
        stack: &mut Vec<NodeId>,
    ) -> Result<(), String> {
        self.pos += 2;
        self.skip_ascii_whitespace();
        if self.pos >= self.bytes.len() {
            return Err("unexpected end of input in closing tag".to_string());
        }

        if !self
            .current_byte()
            .map(is_simple_name_byte)
            .unwrap_or(false)
        {
            return Err(format!("invalid closing tag at byte {}", self.pos));
        }

        let closing_name = self.parse_name_token("closing tag")?;
        self.skip_ascii_whitespace();
        if self.current_byte() != Some(b'>') {
            return Err(format!(
                "expected `>` to close `</{}>` at byte {}",
                closing_name, self.pos
            ));
        }
        self.pos += 1;

        if stack.len() == 1 {
            return Err(format!("unexpected closing tag </{}>", closing_name));
        }

        let open_id = stack.pop().expect("stack length checked above");
        let open_name = store.tag_name_for(open_id).unwrap_or("unknown").to_string();
        if open_name != closing_name {
            return Err(format!(
                "mismatched closing tag </{}>, expected </{}>",
                closing_name, open_name
            ));
        }

        Ok(())
    }

    fn parse_name_token(&mut self, kind: &str) -> Result<String, String> {
        let start = self.pos;
        while let Some(byte) = self.current_byte() {
            if is_simple_name_byte(byte) {
                self.pos += 1;
            } else {
                break;
            }
        }

        if self.pos == start {
            return Err(format!("expected {} name at byte {}", kind, start));
        }

        Ok(self.input[start..self.pos].to_ascii_lowercase())
    }

    fn parse_attribute_value(&mut self) -> Result<String, String> {
        match self.current_byte() {
            Some(quote @ b'"') | Some(quote @ b'\'') => {
                self.pos += 1;
                let rest = &self.bytes[self.pos..];
                let end = rest
                    .iter()
                    .position(|byte| *byte == quote)
                    .ok_or_else(|| format!("unterminated quoted attribute at byte {}", self.pos))?;
                let value = &self.input[self.pos..self.pos + end];
                self.pos += end + 1;
                Ok(value.to_string())
            }
            Some(_) => {
                let start = self.pos;
                while let Some(byte) = self.current_byte() {
                    if byte.is_ascii_whitespace() || byte == b'>' {
                        break;
                    }
                    self.pos += 1;
                }

                if self.pos == start {
                    return Err(format!("expected attribute value at byte {}", start));
                }

                Ok(self.input[start..self.pos].to_string())
            }
            None => Err("unexpected end of input while parsing attribute value".to_string()),
        }
    }
}

fn write_indent(output: &mut String, indent: usize) {
    for _ in 0..indent {
        output.push_str("  ");
    }
}

fn format_attributes(attributes: &BTreeMap<String, String>) -> String {
    let mut parts = Vec::new();
    for (name, value) in attributes {
        if value.is_empty() {
            parts.push(name.clone());
        } else {
            parts.push(format!(r#"{name}="{}""#, escape_attr(value)));
        }
    }
    parts.join(" ")
}

fn escape_text(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn escape_attr(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn is_simple_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_')
}

fn element_namespace_for_root(tag_name: &str) -> &'static str {
    match tag_name {
        "svg" => SVG_NAMESPACE_URI,
        "math" => MATHML_NAMESPACE_URI,
        _ => HTML_NAMESPACE_URI,
    }
}

fn supports_disabled_pseudo_class(tag_name: &str) -> bool {
    matches!(
        tag_name,
        "button" | "fieldset" | "input" | "option" | "optgroup" | "select" | "textarea"
    )
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

fn is_text_input_type(input_type: Option<&str>) -> bool {
    matches!(
        input_type.unwrap_or("text"),
        "text"
            | "search"
            | "url"
            | "tel"
            | "email"
            | "password"
            | "number"
            | "date"
            | "datetime-local"
            | "month"
            | "week"
            | "time"
            | "color"
    )
}

fn is_checkable_input_type(input_type: Option<&str>) -> bool {
    matches!(input_type.unwrap_or("text"), "checkbox" | "radio")
}

fn is_file_input_type(input_type: Option<&str>) -> bool {
    matches!(input_type.unwrap_or("text"), "file")
}
