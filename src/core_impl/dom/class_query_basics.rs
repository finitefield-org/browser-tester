use super::*;

impl Dom {
    pub(crate) fn class_contains(&self, node_id: NodeId, class_name: &str) -> Result<bool> {
        let element = self
            .element(node_id)
            .ok_or_else(|| Error::ScriptRuntime("classList target is not an element".into()))?;
        Ok(has_class(element, class_name))
    }

    pub(crate) fn class_add(&mut self, node_id: NodeId, class_name: &str) -> Result<()> {
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

    pub(crate) fn class_remove(&mut self, node_id: NodeId, class_name: &str) -> Result<()> {
        let element = self
            .element_mut(node_id)
            .ok_or_else(|| Error::ScriptRuntime("classList target is not an element".into()))?;
        let mut classes = class_tokens(element.attrs.get("class").map(String::as_str));
        classes.retain(|name| name != class_name);
        set_class_attr(element, &classes);
        Ok(())
    }

    pub(crate) fn class_toggle(&mut self, node_id: NodeId, class_name: &str) -> Result<bool> {
        let has = self.class_contains(node_id, class_name)?;
        if has {
            self.class_remove(node_id, class_name)?;
            Ok(false)
        } else {
            self.class_add(node_id, class_name)?;
            Ok(true)
        }
    }

    pub(crate) fn class_replace(
        &mut self,
        node_id: NodeId,
        old_class_name: &str,
        new_class_name: &str,
    ) -> Result<bool> {
        let element = self
            .element_mut(node_id)
            .ok_or_else(|| Error::ScriptRuntime("classList target is not an element".into()))?;
        let classes = class_tokens(element.attrs.get("class").map(String::as_str));
        if !classes.iter().any(|name| name == old_class_name) {
            return Ok(false);
        }
        if old_class_name == new_class_name {
            return Ok(true);
        }

        let mut next = Vec::with_capacity(classes.len());
        let mut replaced = false;
        for class_name in classes {
            if class_name == old_class_name && !replaced {
                replaced = true;
                if !next.iter().any(|name| name == new_class_name) {
                    next.push(new_class_name.to_string());
                }
                continue;
            }
            if class_name == old_class_name {
                continue;
            }
            if !next.iter().any(|name| name == &class_name) {
                next.push(class_name);
            }
        }
        set_class_attr(element, &next);
        Ok(true)
    }

    pub(crate) fn query_selector(&self, selector: &str) -> Result<Option<NodeId>> {
        let all = self.query_selector_all(selector)?;
        Ok(all.into_iter().next())
    }

    pub(crate) fn query_selector_all(&self, selector: &str) -> Result<Vec<NodeId>> {
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

    pub(crate) fn query_selector_from(
        &self,
        root: &NodeId,
        selector: &str,
    ) -> Result<Option<NodeId>> {
        let all = self.query_selector_all_from(root, selector)?;
        Ok(all.into_iter().next())
    }

    pub(crate) fn query_selector_all_from(
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
                .any(|steps| self.matches_selector_chain_in_scope(candidate, steps, Some(*root)))
                && seen.insert(candidate)
            {
                matched.push(candidate);
            }
        }
        Ok(matched)
    }

    pub(crate) fn get_elements_by_class_names_from(
        &self,
        root: &NodeId,
        class_names: &[String],
    ) -> Vec<NodeId> {
        if class_names.is_empty() {
            return Vec::new();
        }

        let mut ids = Vec::new();
        self.collect_elements_descendants_dfs(*root, &mut ids);

        ids.into_iter()
            .filter(|node_id| {
                self.element(*node_id).is_some_and(|element| {
                    class_names
                        .iter()
                        .all(|class_name| has_class(element, class_name))
                })
            })
            .collect()
    }

    pub(crate) fn get_elements_by_name_from(&self, root: &NodeId, name: &str) -> Vec<NodeId> {
        let mut ids = Vec::new();
        self.collect_elements_descendants_dfs(*root, &mut ids);

        ids.into_iter()
            .filter(|node_id| {
                self.element(*node_id).is_some_and(|element| {
                    element.attrs.get("name").is_some_and(|value| value == name)
                })
            })
            .collect()
    }

    pub(crate) fn get_elements_by_tag_name_from(
        &self,
        root: &NodeId,
        tag_name: &str,
    ) -> Vec<NodeId> {
        let mut ids = Vec::new();
        self.collect_elements_descendants_dfs(*root, &mut ids);

        if tag_name == "*" {
            return ids;
        }

        ids.into_iter()
            .filter(|node_id| {
                self.tag_name(*node_id)
                    .is_some_and(|candidate| candidate.eq_ignore_ascii_case(tag_name))
            })
            .collect()
    }

    pub(crate) fn get_elements_by_tag_name_ns_from(
        &self,
        root: &NodeId,
        namespace_uri: Option<&str>,
        local_name: &str,
    ) -> Vec<NodeId> {
        let mut ids = Vec::new();
        self.collect_elements_descendants_dfs(*root, &mut ids);

        ids.into_iter()
            .filter(|node_id| {
                self.element(*node_id).is_some_and(|element| {
                    let namespace_matches = match namespace_uri {
                        Some("*") => true,
                        Some(expected) => element.namespace_uri.as_deref() == Some(expected),
                        None => element.namespace_uri.is_none(),
                    };
                    if !namespace_matches {
                        return false;
                    }

                    if local_name == "*" {
                        return true;
                    }

                    let candidate_local_name = element
                        .tag_name
                        .rsplit_once(':')
                        .map(|(_, local)| local)
                        .unwrap_or(element.tag_name.as_str());
                    candidate_local_name == local_name
                })
            })
            .collect()
    }

    pub(crate) fn matches_selector(&self, node_id: NodeId, selector: &str) -> Result<bool> {
        if self.element(node_id).is_none() {
            return Ok(false);
        }

        let groups = parse_selector_groups(selector)?;
        Ok(groups
            .iter()
            .any(|steps| self.matches_selector_chain_in_scope(node_id, steps, Some(node_id))))
    }

    pub(crate) fn closest(&self, node_id: NodeId, selector: &str) -> Result<Option<NodeId>> {
        if self.element(node_id).is_none() {
            return Ok(None);
        }

        let groups = parse_selector_groups(selector)?;
        let mut cursor = Some(node_id);
        while let Some(current) = cursor {
            if groups
                .iter()
                .any(|steps| self.matches_selector_chain_in_scope(current, steps, Some(node_id)))
            {
                return Ok(Some(current));
            }
            cursor = self.parent(current);
        }
        Ok(None)
    }
}
