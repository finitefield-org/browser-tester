use std::collections::BTreeMap;
use std::fmt::Write as _;

use super::DomStore;
use super::DomIndexes;
use super::ElementData;
use super::NodeId;
use super::NodeKind;
use super::NodeRecord;
use super::TextData;

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

        if let Some(id_selector) = selector.strip_prefix('#') {
            return self.select_by_id(id_selector);
        }

        if selector.starts_with('[') {
            return self.select_by_attribute(selector);
        }

        self.select_by_tag(selector)
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
            "input" if is_checkable_input_type(element.attributes.get("type").map(String::as_str)) => {
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
        let node_id = self.add_node(
            parent,
            NodeKind::Element(ElementData {
                tag_name: tag_name.clone(),
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

    fn add_text(&mut self, parent: NodeId, value: String) -> NodeId {
        self.add_node(parent, NodeKind::Text(TextData { value }))
    }

    fn add_comment(&mut self, parent: NodeId, value: String) -> NodeId {
        self.add_node(parent, NodeKind::Comment(value))
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
                            value: element
                                .attributes
                                .get("value")
                                .cloned()
                                .unwrap_or_default(),
                            checked: false,
                        },
                    );
                }
                "input"
                    if is_checkable_input_type(element.attributes.get("type").map(String::as_str)) =>
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

    fn select_by_id(&self, id_selector: &str) -> Result<Vec<NodeId>, String> {
        let id_selector = id_selector.trim();
        if !is_simple_selector_token(id_selector) {
            return Err(format!(
                "unsupported selector `{}`; supported forms are #id, tag, and [attr]",
                format!("#{}", id_selector)
            ));
        }

        Ok(self
            .indexes
            .id_index
            .get(id_selector)
            .copied()
            .into_iter()
            .collect())
    }

    fn select_by_tag(&self, tag_selector: &str) -> Result<Vec<NodeId>, String> {
        if !is_simple_selector_token(tag_selector) {
            return Err(format!(
                "unsupported selector `{}`; supported forms are #id, tag, and [attr]",
                tag_selector
            ));
        }

        let tag = tag_selector.to_ascii_lowercase();
        Ok(self
            .indexes
            .tag_index
            .get(&tag)
            .cloned()
            .unwrap_or_default())
    }

    fn select_by_attribute(&self, selector: &str) -> Result<Vec<NodeId>, String> {
        if !selector.ends_with(']') {
            return Err(format!(
                "unsupported selector `{}`; supported forms are #id, tag, and [attr]",
                selector
            ));
        }

        let attribute = selector[1..selector.len() - 1].trim();
        if !is_simple_selector_token(attribute) {
            return Err(format!(
                "unsupported selector `{}`; supported forms are #id, tag, and [attr]",
                selector
            ));
        }

        let attribute = attribute.to_ascii_lowercase();
        Ok(self
            .nodes
            .iter()
            .filter_map(|node| match &node.kind {
                NodeKind::Element(element) if element.attributes.contains_key(&attribute) => {
                    Some(node.id)
                }
                _ => None,
            })
            .collect())
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

fn is_simple_selector_token(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(is_simple_name_byte)
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
