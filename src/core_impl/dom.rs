use super::form_controls::is_radio_input;
use super::html::{is_void_tag, parse_html};
use super::*;

fn is_checkbox_or_radio_input_element(element: &Element) -> bool {
    if !element.tag_name.eq_ignore_ascii_case("input") {
        return false;
    }
    matches!(
        element
            .attrs
            .get("type")
            .map(|kind| kind.to_ascii_lowercase())
            .as_deref(),
        Some("checkbox") | Some("radio")
    )
}

impl Dom {
    pub(super) fn new() -> Self {
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

    pub(super) fn create_node(&mut self, parent: Option<NodeId>, node_type: NodeType) -> NodeId {
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

    pub(super) fn create_element(
        &mut self,
        parent: NodeId,
        tag_name: String,
        attrs: HashMap<String, String>,
    ) -> NodeId {
        let value = attrs.get("value").cloned().unwrap_or_default();
        let value_len = value.chars().count();
        let checked = attrs.contains_key("checked");
        let disabled = attrs.contains_key("disabled");
        let readonly = attrs.contains_key("readonly");
        let required = attrs.contains_key("required");
        let element = Element {
            tag_name,
            attrs,
            value,
            checked,
            indeterminate: false,
            disabled,
            readonly,
            required,
            custom_validity_message: String::new(),
            selection_start: value_len,
            selection_end: value_len,
            selection_direction: "none".to_string(),
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

    pub(super) fn create_detached_element(&mut self, tag_name: String) -> NodeId {
        let element = Element {
            tag_name,
            attrs: HashMap::new(),
            value: String::new(),
            checked: false,
            indeterminate: false,
            disabled: false,
            readonly: false,
            required: false,
            custom_validity_message: String::new(),
            selection_start: 0,
            selection_end: 0,
            selection_direction: "none".to_string(),
        };
        self.create_node(None, NodeType::Element(element))
    }

    pub(super) fn create_detached_text(&mut self, text: String) -> NodeId {
        self.create_node(None, NodeType::Text(text))
    }

    pub(super) fn create_text(&mut self, parent: NodeId, text: String) -> NodeId {
        self.create_node(Some(parent), NodeType::Text(text))
    }

    pub(super) fn element(&self, node_id: NodeId) -> Option<&Element> {
        match &self.nodes[node_id.0].node_type {
            NodeType::Element(element) => Some(element),
            _ => None,
        }
    }

    pub(super) fn element_mut(&mut self, node_id: NodeId) -> Option<&mut Element> {
        match &mut self.nodes[node_id.0].node_type {
            NodeType::Element(element) => Some(element),
            _ => None,
        }
    }

    pub(super) fn tag_name(&self, node_id: NodeId) -> Option<&str> {
        self.element(node_id).map(|e| e.tag_name.as_str())
    }

    pub(super) fn parent(&self, node_id: NodeId) -> Option<NodeId> {
        self.nodes[node_id.0].parent
    }

    pub(super) fn is_descendant_of(&self, node_id: NodeId, ancestor: NodeId) -> bool {
        let mut cursor = self.parent(node_id);
        while let Some(current) = cursor {
            if current == ancestor {
                return true;
            }
            cursor = self.parent(current);
        }
        false
    }

    pub(super) fn active_element(&self) -> Option<NodeId> {
        self.active_element
    }

    pub(super) fn set_active_element(&mut self, node: Option<NodeId>) {
        self.active_element = node;
    }

    pub(super) fn active_pseudo_element(&self) -> Option<NodeId> {
        self.active_pseudo_element
    }

    pub(super) fn set_active_pseudo_element(&mut self, node: Option<NodeId>) {
        self.active_pseudo_element = node;
    }

    pub(super) fn by_id(&self, id: &str) -> Option<NodeId> {
        self.id_index.get(id).and_then(|ids| ids.first().copied())
    }

    pub(super) fn by_id_all(&self, id: &str) -> Vec<NodeId> {
        self.id_index.get(id).cloned().unwrap_or_default()
    }

    pub(super) fn index_id(&mut self, id: &str, node_id: NodeId) {
        if id.is_empty() {
            return;
        }
        self.id_index
            .entry(id.to_string())
            .or_default()
            .push(node_id);
    }

    pub(super) fn unindex_id(&mut self, id: &str, node_id: NodeId) {
        let Some(nodes) = self.id_index.get_mut(id) else {
            return;
        };
        nodes.retain(|candidate| *candidate != node_id);
        if nodes.is_empty() {
            self.id_index.remove(id);
        }
    }

    pub(super) fn text_content(&self, node_id: NodeId) -> String {
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

    pub(super) fn set_text_content(&mut self, node_id: NodeId, value: &str) -> Result<()> {
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

    pub(super) fn inner_html(&self, node_id: NodeId) -> Result<String> {
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

    pub(super) fn outer_html(&self, node_id: NodeId) -> Result<String> {
        if self.element(node_id).is_none() {
            return Err(Error::ScriptRuntime(
                "outerHTML target is not an element".into(),
            ));
        }
        Ok(self.dump_node(node_id))
    }

    pub(super) fn set_inner_html(&mut self, node_id: NodeId, html: &str) -> Result<()> {
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
            let _ = self.clone_subtree_from_dom(&fragment, child, Some(node_id), true)?;
        }

        self.rebuild_id_index();
        Ok(())
    }

    pub(super) fn set_outer_html(&mut self, node_id: NodeId, html: &str) -> Result<()> {
        if self.element(node_id).is_none() {
            return Err(Error::ScriptRuntime(
                "outerHTML target is not an element".into(),
            ));
        }

        let Some(parent) = self.parent(node_id) else {
            return Err(Error::ScriptRuntime("outerHTML target is detached".into()));
        };

        let index = self.nodes[parent.0]
            .children
            .iter()
            .position(|id| *id == node_id)
            .ok_or_else(|| Error::ScriptRuntime("outerHTML target is detached".into()))?;

        let ParseOutput { dom: fragment, .. } = parse_html(html)?;

        self.nodes[parent.0].children.remove(index);
        self.nodes[node_id.0].parent = None;

        let mut insert_at = index;
        let children = fragment.nodes[fragment.root.0].children.clone();
        for child in children {
            if let Some(cloned) = self.clone_subtree_from_dom(&fragment, child, None, true)? {
                self.nodes[cloned.0].parent = Some(parent);
                self.nodes[parent.0].children.insert(insert_at, cloned);
                insert_at += 1;
            }
        }

        self.rebuild_id_index();
        Ok(())
    }

    pub(super) fn insert_adjacent_html(
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
            if let Some(node) = self.clone_subtree_from_dom(&fragment, child, None, true)? {
                self.insert_adjacent_node(target, position, node)?;
            }
        }
        Ok(())
    }

    pub(super) fn clone_subtree_from_dom(
        &mut self,
        source: &Dom,
        source_node: NodeId,
        parent: Option<NodeId>,
        sanitize: bool,
    ) -> Result<Option<NodeId>> {
        let node_type = match &source.nodes[source_node.0].node_type {
            NodeType::Document => {
                return Err(Error::ScriptRuntime(
                    "cannot clone document node into innerHTML target".into(),
                ));
            }
            NodeType::Element(element) => {
                if sanitize && should_strip_inner_html_element(&element.tag_name) {
                    return Ok(None);
                }
                let mut clone = element.clone();
                if sanitize {
                    sanitize_inner_html_element_attrs(&mut clone);
                }
                NodeType::Element(clone)
            }
            NodeType::Text(text) => NodeType::Text(text.clone()),
        };

        let node = self.create_node(parent, node_type);
        for child in &source.nodes[source_node.0].children {
            let _ = self.clone_subtree_from_dom(source, *child, Some(node), sanitize)?;
        }
        Ok(Some(node))
    }

    pub(super) fn value(&self, node_id: NodeId) -> Result<String> {
        let element = self
            .element(node_id)
            .ok_or_else(|| Error::ScriptRuntime("value target is not an element".into()))?;
        if is_checkbox_or_radio_input_element(element) && !element.attrs.contains_key("value") {
            return Ok("on".to_string());
        }
        Ok(element.value.clone())
    }

    pub(super) fn set_value(&mut self, node_id: NodeId, value: &str) -> Result<()> {
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
        if is_checkbox_or_radio_input_element(element) {
            element.attrs.insert("value".to_string(), value.to_string());
        }
        element.value = value.to_string();
        let len = element.value.chars().count();
        element.selection_start = len;
        element.selection_end = len;
        element.selection_direction = "none".to_string();
        Ok(())
    }

    pub(super) fn indeterminate(&self, node_id: NodeId) -> Result<bool> {
        let element = self
            .element(node_id)
            .ok_or_else(|| Error::ScriptRuntime("indeterminate target is not an element".into()))?;
        Ok(element.indeterminate)
    }

    pub(super) fn set_indeterminate(&mut self, node_id: NodeId, indeterminate: bool) -> Result<()> {
        let element = self
            .element_mut(node_id)
            .ok_or_else(|| Error::ScriptRuntime("indeterminate target is not an element".into()))?;
        element.indeterminate = indeterminate;
        Ok(())
    }

    pub(super) fn custom_validity_message(&self, node_id: NodeId) -> Result<String> {
        let element = self.element(node_id).ok_or_else(|| {
            Error::ScriptRuntime("custom validity target is not an element".into())
        })?;
        Ok(element.custom_validity_message.clone())
    }

    pub(super) fn set_custom_validity_message(
        &mut self,
        node_id: NodeId,
        message: &str,
    ) -> Result<()> {
        let element = self.element_mut(node_id).ok_or_else(|| {
            Error::ScriptRuntime("custom validity target is not an element".into())
        })?;
        element.custom_validity_message = message.to_string();
        Ok(())
    }

    pub(super) fn selection_start(&self, node_id: NodeId) -> Result<usize> {
        let element = self.element(node_id).ok_or_else(|| {
            Error::ScriptRuntime("selectionStart target is not an element".into())
        })?;
        Ok(element.selection_start)
    }

    pub(super) fn selection_end(&self, node_id: NodeId) -> Result<usize> {
        let element = self
            .element(node_id)
            .ok_or_else(|| Error::ScriptRuntime("selectionEnd target is not an element".into()))?;
        Ok(element.selection_end)
    }

    pub(super) fn selection_direction(&self, node_id: NodeId) -> Result<String> {
        let element = self.element(node_id).ok_or_else(|| {
            Error::ScriptRuntime("selectionDirection target is not an element".into())
        })?;
        Ok(element.selection_direction.clone())
    }

    pub(super) fn set_selection_range(
        &mut self,
        node_id: NodeId,
        start: usize,
        end: usize,
        direction: &str,
    ) -> Result<()> {
        let element = self.element_mut(node_id).ok_or_else(|| {
            Error::ScriptRuntime("setSelectionRange target is not an element".into())
        })?;
        let len = element.value.chars().count();
        let clamped_start = start.min(len);
        let clamped_end = end.min(len);
        let (selection_start, selection_end) = if clamped_end < clamped_start {
            (clamped_end, clamped_end)
        } else {
            (clamped_start, clamped_end)
        };
        element.selection_start = selection_start;
        element.selection_end = selection_end;
        element.selection_direction = direction.to_string();
        Ok(())
    }

    pub(super) fn initialize_form_control_values(&mut self) -> Result<()> {
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
                let len = element.value.chars().count();
                element.selection_start = len;
                element.selection_end = len;
                element.selection_direction = "none".to_string();
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

    pub(super) fn sync_select_value_for_option(&mut self, option_node: NodeId) -> Result<()> {
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

    pub(super) fn set_select_value(&mut self, select_node: NodeId, requested: &str) -> Result<()> {
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

    pub(super) fn sync_select_value(&mut self, select_node: NodeId) -> Result<()> {
        let value = self.select_value_from_options(select_node)?;
        let element = self
            .element_mut(select_node)
            .ok_or_else(|| Error::ScriptRuntime("select target is not an element".into()))?;
        element.value = value;
        Ok(())
    }

    pub(super) fn select_value_from_options(&self, select_node: NodeId) -> Result<String> {
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

    pub(super) fn collect_select_options(&self, node: NodeId, out: &mut Vec<NodeId>) {
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

    pub(super) fn option_effective_value(&self, option_node: NodeId) -> Result<String> {
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

    pub(super) fn checked(&self, node_id: NodeId) -> Result<bool> {
        let element = self
            .element(node_id)
            .ok_or_else(|| Error::ScriptRuntime("checked target is not an element".into()))?;
        Ok(element.checked)
    }

    pub(super) fn set_checked(&mut self, node_id: NodeId, checked: bool) -> Result<()> {
        if checked && is_radio_input(self, node_id) {
            self.uncheck_other_radios_in_group(node_id);
        }
        let element = self
            .element_mut(node_id)
            .ok_or_else(|| Error::ScriptRuntime("checked target is not an element".into()))?;
        element.checked = checked;
        Ok(())
    }

    pub(super) fn uncheck_other_radios_in_group(&mut self, target: NodeId) {
        let target_name = self.attr(target, "name").unwrap_or_default();
        if target_name.is_empty() {
            return;
        }
        let target_form = self.find_ancestor_by_tag(target, "form");

        let all_nodes = self.all_element_nodes();
        for node in all_nodes {
            if node == target {
                continue;
            }
            if !is_radio_input(self, node) {
                continue;
            }
            if self.attr(node, "name").unwrap_or_default() != target_name {
                continue;
            }
            if self.find_ancestor_by_tag(node, "form") != target_form {
                continue;
            }
            if let Some(element) = self.element_mut(node) {
                element.checked = false;
            }
        }
    }

    pub(super) fn normalize_radio_groups(&mut self) -> Result<()> {
        let all_nodes = self.all_element_nodes();
        for node in all_nodes {
            if !is_radio_input(self, node) {
                continue;
            }
            if self.attr(node, "checked").is_some() {
                self.set_checked(node, true)?;
            }
        }
        Ok(())
    }

    pub(super) fn disabled(&self, node_id: NodeId) -> bool {
        self.element(node_id).map(|e| e.disabled).unwrap_or(false)
    }

    pub(super) fn readonly(&self, node_id: NodeId) -> bool {
        self.element(node_id).map(|e| e.readonly).unwrap_or(false)
    }

    pub(super) fn required(&self, node_id: NodeId) -> bool {
        self.element(node_id).map(|e| e.required).unwrap_or(false)
    }

    pub(super) fn attr(&self, node_id: NodeId, name: &str) -> Option<String> {
        self.element(node_id)
            .and_then(|e| e.attrs.get(name).cloned())
    }

    pub(super) fn has_attr(&self, node_id: NodeId, name: &str) -> Result<bool> {
        let element = self
            .element(node_id)
            .ok_or_else(|| Error::ScriptRuntime("hasAttribute target is not an element".into()))?;
        Ok(element.attrs.contains_key(&name.to_ascii_lowercase()))
    }

    pub(super) fn set_attr(&mut self, node_id: NodeId, name: &str, value: &str) -> Result<()> {
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
                let len = element.value.chars().count();
                element.selection_start = len;
                element.selection_end = len;
                element.selection_direction = "none".to_string();
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

    pub(super) fn remove_attr(&mut self, node_id: NodeId, name: &str) -> Result<()> {
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
                element.selection_start = 0;
                element.selection_end = 0;
                element.selection_direction = "none".to_string();
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

    pub(super) fn append_child(&mut self, parent: NodeId, child: NodeId) -> Result<()> {
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

    pub(super) fn prepend_child(&mut self, parent: NodeId, child: NodeId) -> Result<()> {
        let reference = self.nodes[parent.0].children.first().copied();
        if let Some(reference) = reference {
            self.insert_before(parent, child, reference)
        } else {
            self.append_child(parent, child)
        }
    }

    pub(super) fn insert_before(
        &mut self,
        parent: NodeId,
        child: NodeId,
        reference: NodeId,
    ) -> Result<()> {
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

    pub(super) fn insert_after(&mut self, target: NodeId, child: NodeId) -> Result<()> {
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

    pub(super) fn replace_with(&mut self, target: NodeId, child: NodeId) -> Result<()> {
        let Some(parent) = self.parent(target) else {
            return Ok(());
        };
        if target == child {
            return Ok(());
        }
        self.insert_before(parent, child, target)?;
        self.remove_child(parent, target)
    }

    pub(super) fn insert_adjacent_node(
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

    pub(super) fn remove_child(&mut self, parent: NodeId, child: NodeId) -> Result<()> {
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

    pub(super) fn remove_node(&mut self, node: NodeId) -> Result<()> {
        if node == self.root {
            return Err(Error::ScriptRuntime("cannot remove document root".into()));
        }
        let Some(parent) = self.parent(node) else {
            return Ok(());
        };
        self.remove_child(parent, node)
    }

    pub(super) fn dataset_get(&self, node_id: NodeId, key: &str) -> Result<String> {
        if self.element(node_id).is_none() {
            return Err(Error::ScriptRuntime(
                "dataset target is not an element".into(),
            ));
        }
        let name = dataset_key_to_attr_name(key);
        Ok(self.attr(node_id, &name).unwrap_or_default())
    }

    pub(super) fn dataset_set(&mut self, node_id: NodeId, key: &str, value: &str) -> Result<()> {
        let name = dataset_key_to_attr_name(key);
        self.set_attr(node_id, &name, value)
    }

    pub(super) fn style_get(&self, node_id: NodeId, key: &str) -> Result<String> {
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

    pub(super) fn style_set(&mut self, node_id: NodeId, key: &str, value: &str) -> Result<()> {
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

    pub(super) fn offset_left(&self, node_id: NodeId) -> Result<i64> {
        if self.element(node_id).is_none() {
            return Err(Error::ScriptRuntime(
                "offsetLeft target is not an element".into(),
            ));
        }
        Ok(0)
    }

    pub(super) fn offset_top(&self, node_id: NodeId) -> Result<i64> {
        if self.element(node_id).is_none() {
            return Err(Error::ScriptRuntime(
                "offsetTop target is not an element".into(),
            ));
        }
        Ok(0)
    }

    pub(super) fn offset_width(&self, node_id: NodeId) -> Result<i64> {
        if self.element(node_id).is_none() {
            return Err(Error::ScriptRuntime(
                "offsetWidth target is not an element".into(),
            ));
        }
        Ok(0)
    }

    pub(super) fn offset_height(&self, node_id: NodeId) -> Result<i64> {
        if self.element(node_id).is_none() {
            return Err(Error::ScriptRuntime(
                "offsetHeight target is not an element".into(),
            ));
        }
        Ok(0)
    }

    pub(super) fn scroll_width(&self, node_id: NodeId) -> Result<i64> {
        if self.element(node_id).is_none() {
            return Err(Error::ScriptRuntime(
                "scrollWidth target is not an element".into(),
            ));
        }
        Ok(0)
    }

    pub(super) fn scroll_height(&self, node_id: NodeId) -> Result<i64> {
        if self.element(node_id).is_none() {
            return Err(Error::ScriptRuntime(
                "scrollHeight target is not an element".into(),
            ));
        }
        Ok(0)
    }

    pub(super) fn scroll_left(&self, node_id: NodeId) -> Result<i64> {
        if self.element(node_id).is_none() {
            return Err(Error::ScriptRuntime(
                "scrollLeft target is not an element".into(),
            ));
        }
        Ok(0)
    }

    pub(super) fn scroll_top(&self, node_id: NodeId) -> Result<i64> {
        if self.element(node_id).is_none() {
            return Err(Error::ScriptRuntime(
                "scrollTop target is not an element".into(),
            ));
        }
        Ok(0)
    }

    pub(super) fn class_contains(&self, node_id: NodeId, class_name: &str) -> Result<bool> {
        let element = self
            .element(node_id)
            .ok_or_else(|| Error::ScriptRuntime("classList target is not an element".into()))?;
        Ok(has_class(element, class_name))
    }

    pub(super) fn class_add(&mut self, node_id: NodeId, class_name: &str) -> Result<()> {
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

    pub(super) fn class_remove(&mut self, node_id: NodeId, class_name: &str) -> Result<()> {
        let element = self
            .element_mut(node_id)
            .ok_or_else(|| Error::ScriptRuntime("classList target is not an element".into()))?;
        let mut classes = class_tokens(element.attrs.get("class").map(String::as_str));
        classes.retain(|name| name != class_name);
        set_class_attr(element, &classes);
        Ok(())
    }

    pub(super) fn class_toggle(&mut self, node_id: NodeId, class_name: &str) -> Result<bool> {
        let has = self.class_contains(node_id, class_name)?;
        if has {
            self.class_remove(node_id, class_name)?;
            Ok(false)
        } else {
            self.class_add(node_id, class_name)?;
            Ok(true)
        }
    }

    pub(super) fn query_selector(&self, selector: &str) -> Result<Option<NodeId>> {
        let all = self.query_selector_all(selector)?;
        Ok(all.into_iter().next())
    }

    pub(super) fn query_selector_all(&self, selector: &str) -> Result<Vec<NodeId>> {
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

    pub(super) fn query_selector_from(
        &self,
        root: &NodeId,
        selector: &str,
    ) -> Result<Option<NodeId>> {
        let all = self.query_selector_all_from(root, selector)?;
        Ok(all.into_iter().next())
    }

    pub(super) fn query_selector_all_from(
        &self,
        root: &NodeId,
        selector: &str,
    ) -> Result<Vec<NodeId>> {
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

    pub(super) fn matches_selector(&self, node_id: NodeId, selector: &str) -> Result<bool> {
        if self.element(node_id).is_none() {
            return Ok(false);
        }

        let groups = parse_selector_groups(selector)?;
        Ok(groups
            .iter()
            .any(|steps| self.matches_selector_chain(node_id, steps)))
    }

    pub(super) fn closest(&self, node_id: NodeId, selector: &str) -> Result<Option<NodeId>> {
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

    pub(super) fn can_have_children(&self, node_id: NodeId) -> bool {
        matches!(
            self.nodes.get(node_id.0).map(|n| &n.node_type),
            Some(NodeType::Document | NodeType::Element(_))
        )
    }

    pub(super) fn is_valid_node(&self, node_id: NodeId) -> bool {
        node_id.0 < self.nodes.len()
    }

    pub(super) fn is_connected(&self, node_id: NodeId) -> bool {
        let mut cursor = Some(node_id);
        while let Some(node) = cursor {
            if node == self.root {
                return true;
            }
            cursor = self.parent(node);
        }
        false
    }

    pub(super) fn rebuild_id_index(&mut self) {
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

    pub(super) fn index_id_map(next: &mut HashMap<String, Vec<NodeId>>, id: &str, node_id: NodeId) {
        if id.is_empty() {
            return;
        }
        next.entry(id.to_string()).or_default().push(node_id);
    }

    pub(super) fn collect_elements_dfs(&self, node_id: NodeId, out: &mut Vec<NodeId>) {
        if matches!(self.nodes[node_id.0].node_type, NodeType::Element(_)) {
            out.push(node_id);
        }
        for child in &self.nodes[node_id.0].children {
            self.collect_elements_dfs(*child, out);
        }
    }

    pub(super) fn collect_elements_descendants_dfs(&self, node_id: NodeId, out: &mut Vec<NodeId>) {
        for child in &self.nodes[node_id.0].children {
            self.collect_elements_dfs(*child, out);
        }
    }

    pub(super) fn all_element_nodes(&self) -> Vec<NodeId> {
        let mut out = Vec::new();
        self.collect_elements_dfs(self.root, &mut out);
        out
    }

    pub(super) fn child_elements(&self, node_id: NodeId) -> Vec<NodeId> {
        self.nodes[node_id.0]
            .children
            .iter()
            .copied()
            .filter(|child| self.element(*child).is_some())
            .collect()
    }

    pub(super) fn child_element_count(&self, node_id: NodeId) -> usize {
        self.child_elements(node_id).len()
    }

    pub(super) fn first_element_child(&self, node_id: NodeId) -> Option<NodeId> {
        self.nodes[node_id.0]
            .children
            .iter()
            .copied()
            .find(|child| self.element(*child).is_some())
    }

    pub(super) fn last_element_child(&self, node_id: NodeId) -> Option<NodeId> {
        self.nodes[node_id.0]
            .children
            .iter()
            .rev()
            .copied()
            .find(|child| self.element(*child).is_some())
    }

    pub(super) fn document_element(&self) -> Option<NodeId> {
        self.first_element_child(self.root)
    }

    pub(super) fn head(&self) -> Option<NodeId> {
        if let Some(document_element) = self.document_element() {
            if self
                .tag_name(document_element)
                .map(|tag| tag.eq_ignore_ascii_case("html"))
                .unwrap_or(false)
            {
                return self
                    .child_elements(document_element)
                    .into_iter()
                    .find(|child| {
                        self.tag_name(*child)
                            .map(|tag| tag.eq_ignore_ascii_case("head"))
                            .unwrap_or(false)
                    });
            }
        }
        self.query_selector("head").ok().flatten()
    }

    pub(super) fn body(&self) -> Option<NodeId> {
        if let Some(document_element) = self.document_element() {
            if self
                .tag_name(document_element)
                .map(|tag| tag.eq_ignore_ascii_case("html"))
                .unwrap_or(false)
            {
                return self
                    .child_elements(document_element)
                    .into_iter()
                    .find(|child| {
                        self.tag_name(*child)
                            .map(|tag| {
                                tag.eq_ignore_ascii_case("body")
                                    || tag.eq_ignore_ascii_case("frameset")
                            })
                            .unwrap_or(false)
                    });
            }
        }
        self.query_selector("body")
            .ok()
            .flatten()
            .or_else(|| self.query_selector("frameset").ok().flatten())
    }

    pub(super) fn document_title(&self) -> String {
        self.query_selector("title")
            .ok()
            .flatten()
            .map(|node| self.text_content(node))
            .unwrap_or_default()
    }

    pub(super) fn set_document_title(&mut self, title: &str) -> Result<()> {
        let title_node = if let Some(existing_title) = self.query_selector("title")? {
            existing_title
        } else {
            let head = self.ensure_head_element()?;
            self.create_element(head, "title".to_string(), HashMap::new())
        };
        self.set_text_content(title_node, title)
    }

    pub(super) fn ensure_head_element(&mut self) -> Result<NodeId> {
        if let Some(head) = self.head() {
            return Ok(head);
        }

        let parent = if let Some(document_element) = self.document_element() {
            if self
                .tag_name(document_element)
                .map(|tag| tag.eq_ignore_ascii_case("html"))
                .unwrap_or(false)
            {
                document_element
            } else {
                self.root
            }
        } else {
            self.create_element(self.root, "html".to_string(), HashMap::new())
        };

        Ok(self.create_element(parent, "head".to_string(), HashMap::new()))
    }

    pub(super) fn matches_selector_chain(&self, node_id: NodeId, steps: &[SelectorPart]) -> bool {
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

    pub(super) fn matches_step(&self, node_id: NodeId, step: &SelectorStep) -> bool {
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
                SelectorPseudoClass::Indeterminate => self
                    .element(node_id)
                    .is_some_and(|node| node.indeterminate && !node.attrs.contains_key("switch")),
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
                SelectorPseudoClass::Is(inners) | SelectorPseudoClass::Where(inners) => inners
                    .iter()
                    .any(|inner| self.matches_selector_chain(node_id, inner)),
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

    pub(super) fn is_first_element_child(&self, node_id: NodeId) -> bool {
        self.previous_element_sibling(node_id).is_none()
    }

    pub(super) fn is_last_element_child(&self, node_id: NodeId) -> bool {
        self.next_element_sibling(node_id).is_none()
    }

    pub(super) fn is_only_element_child(&self, node_id: NodeId) -> bool {
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

    pub(super) fn is_nth_element_child(
        &self,
        node_id: NodeId,
        selector: &NthChildSelector,
    ) -> bool {
        let Some(index) = self.element_index(node_id) else {
            return false;
        };
        self.is_nth_index_element_child(index, selector)
    }

    pub(super) fn is_nth_last_element_child(
        &self,
        node_id: NodeId,
        selector: &NthChildSelector,
    ) -> bool {
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

    pub(super) fn is_nth_element_of_type(
        &self,
        node_id: NodeId,
        selector: &NthChildSelector,
    ) -> bool {
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

    pub(super) fn is_nth_last_element_of_type(
        &self,
        node_id: NodeId,
        selector: &NthChildSelector,
    ) -> bool {
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

    pub(super) fn is_first_of_type(&self, node_id: NodeId) -> bool {
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

    pub(super) fn is_only_of_type(&self, node_id: NodeId) -> bool {
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

    pub(super) fn is_last_of_type(&self, node_id: NodeId) -> bool {
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

    pub(super) fn is_nth_index_element_child(
        &self,
        index: usize,
        selector: &NthChildSelector,
    ) -> bool {
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

    pub(super) fn element_index(&self, node_id: NodeId) -> Option<usize> {
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

    pub(super) fn next_element_sibling(&self, node_id: NodeId) -> Option<NodeId> {
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

    pub(super) fn previous_element_sibling(&self, node_id: NodeId) -> Option<NodeId> {
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

    pub(super) fn find_ancestor_by_tag(&self, node_id: NodeId, tag: &str) -> Option<NodeId> {
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

    pub(super) fn dump_node(&self, node_id: NodeId) -> String {
        match &self.nodes[node_id.0].node_type {
            NodeType::Document => {
                let mut out = String::new();
                for child in &self.nodes[node_id.0].children {
                    out.push_str(&self.dump_node(*child));
                }
                out
            }
            NodeType::Text(text) => escape_html_text_for_serialization(text),
            NodeType::Element(element) => {
                let mut out = String::new();
                out.push('<');
                out.push_str(&element.tag_name);
                let mut attrs = element.attrs.iter().collect::<Vec<_>>();
                attrs.sort_by(|(left, _), (right, _)| left.cmp(right));
                for (k, v) in attrs {
                    out.push(' ');
                    out.push_str(k);
                    out.push_str("=\"");
                    out.push_str(&escape_html_attr_for_serialization(v));
                    out.push('"');
                }
                out.push('>');
                if is_void_tag(&element.tag_name) {
                    return out;
                }
                let raw_text_container = element.tag_name.eq_ignore_ascii_case("script")
                    || element.tag_name.eq_ignore_ascii_case("style");
                for child in &self.nodes[node_id.0].children {
                    if raw_text_container {
                        match &self.nodes[child.0].node_type {
                            NodeType::Text(text) => out.push_str(text),
                            _ => out.push_str(&self.dump_node(*child)),
                        }
                    } else {
                        out.push_str(&self.dump_node(*child));
                    }
                }
                out.push_str("</");
                out.push_str(&element.tag_name);
                out.push('>');
                out
            }
        }
    }
}
