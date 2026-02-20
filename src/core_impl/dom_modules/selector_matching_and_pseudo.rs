use super::*;

impl Dom {
    pub(crate) fn matches_step(&self, node_id: NodeId, step: &SelectorStep) -> bool {
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

    pub(crate) fn is_first_element_child(&self, node_id: NodeId) -> bool {
        self.previous_element_sibling(node_id).is_none()
    }

    pub(crate) fn is_last_element_child(&self, node_id: NodeId) -> bool {
        self.next_element_sibling(node_id).is_none()
    }

    pub(crate) fn is_only_element_child(&self, node_id: NodeId) -> bool {
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

    pub(crate) fn is_nth_element_child(
        &self,
        node_id: NodeId,
        selector: &NthChildSelector,
    ) -> bool {
        let Some(index) = self.element_index(node_id) else {
            return false;
        };
        self.is_nth_index_element_child(index, selector)
    }

    pub(crate) fn is_nth_last_element_child(
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

    pub(crate) fn is_nth_element_of_type(
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

    pub(crate) fn is_nth_last_element_of_type(
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

    pub(crate) fn is_first_of_type(&self, node_id: NodeId) -> bool {
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

    pub(crate) fn is_only_of_type(&self, node_id: NodeId) -> bool {
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

    pub(crate) fn is_last_of_type(&self, node_id: NodeId) -> bool {
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

    pub(crate) fn is_nth_index_element_child(
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

    pub(crate) fn element_index(&self, node_id: NodeId) -> Option<usize> {
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

    pub(crate) fn next_element_sibling(&self, node_id: NodeId) -> Option<NodeId> {
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

    pub(crate) fn previous_element_sibling(&self, node_id: NodeId) -> Option<NodeId> {
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

    pub(crate) fn find_ancestor_by_tag(&self, node_id: NodeId, tag: &str) -> Option<NodeId> {
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
}
