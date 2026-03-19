use super::*;

impl Harness {
    pub(crate) fn dom_string_map_synthesized_keys(
        &self,
        entries: &ObjectValue,
        enumerable_only: bool,
    ) -> Option<Vec<String>> {
        if !Self::is_dom_string_map_object(entries) {
            return None;
        }
        let node = Self::dom_string_map_owner_node(entries)
            .filter(|node| self.dom.element(*node).is_some())?;
        let mut keys = self
            .dataset_entries_for_node(node)
            .into_iter()
            .map(|(key, _)| key)
            .filter(|key| Self::own_property_descriptor_object_from_entries(entries, key).is_none())
            .collect::<Vec<_>>();
        keys.extend(if enumerable_only {
            Self::ordered_enumerable_string_keys(entries)
        } else {
            Self::ordered_visible_string_keys(entries)
        });
        Some(keys)
    }

    pub(crate) fn dom_string_map_synthesized_descriptor_value(
        &self,
        entries: &ObjectValue,
        key: &str,
    ) -> Option<Value> {
        if !Self::is_dom_string_map_object(entries)
            || Self::own_property_descriptor_object_from_entries(entries, key).is_some()
        {
            return None;
        }
        let node = Self::dom_string_map_owner_node(entries)
            .filter(|node| self.dom.element(*node).is_some())?;
        let attr_name = dataset_key_to_attr_name(key);
        let value = self.dom.attr(node, &attr_name)?;
        Some(Self::own_data_property_descriptor_with_attrs(
            Value::String(value),
            true,
            true,
            true,
        ))
    }

    pub(crate) fn class_list_synthesized_keys(
        &self,
        entries: &ObjectValue,
        enumerable_only: bool,
    ) -> Option<Vec<String>> {
        if !Self::is_class_list_object(entries) {
            return None;
        }
        let node = match Self::object_get_entry(entries, INTERNAL_CLASS_LIST_NODE_KEY) {
            Some(Value::Node(node)) => Some(node),
            _ => None,
        }?;
        let classes = class_tokens(self.dom.attr(node, "class").as_deref());
        let mut integer_keys = classes
            .iter()
            .enumerate()
            .map(|(index, _)| (index as u64, index.to_string()))
            .collect::<Vec<_>>();
        let property_keys = if enumerable_only {
            Self::ordered_enumerable_string_keys(entries)
        } else {
            Self::ordered_visible_string_keys(entries)
        };
        for key in &property_keys {
            if let Some(index) = Self::own_property_integer_key(key)
                && !integer_keys.iter().any(|(existing, _)| *existing == index)
            {
                integer_keys.push((index, key.clone()));
            }
        }
        integer_keys.sort_by_key(|(index, _)| *index);
        let mut out = integer_keys
            .into_iter()
            .map(|(_, key)| key)
            .collect::<Vec<_>>();
        if !enumerable_only {
            out.push("length".to_string());
            out.push("value".to_string());
        }
        out.extend(property_keys.into_iter().filter(|key| {
            Self::own_property_integer_key(key).is_none()
                && (enumerable_only || (key != "length" && key != "value"))
        }));
        Some(out)
    }

    pub(crate) fn class_list_synthesized_descriptor_value(
        &self,
        entries: &ObjectValue,
        key: &str,
    ) -> Option<Value> {
        if !Self::is_class_list_object(entries)
            || Self::own_property_descriptor_object_from_entries(entries, key).is_some()
        {
            return None;
        }
        let node = match Self::object_get_entry(entries, INTERNAL_CLASS_LIST_NODE_KEY) {
            Some(Value::Node(node)) => Some(node),
            _ => None,
        }?;
        let classes = class_tokens(self.dom.attr(node, "class").as_deref());
        if key == "length" {
            return Some(Self::own_data_property_descriptor_with_attrs(
                Value::Number(classes.len() as i64),
                true,
                false,
                true,
            ));
        }
        if key == "value" {
            return Some(Self::own_data_property_descriptor_with_attrs(
                Value::String(classes.join(" ")),
                true,
                false,
                true,
            ));
        }
        let index = Self::own_property_integer_key(key)? as usize;
        let class_name = classes.get(index)?.clone();
        Some(Self::own_data_property_descriptor_with_attrs(
            Value::String(class_name),
            true,
            true,
            true,
        ))
    }

    pub(crate) fn named_node_map_synthesized_keys(
        &mut self,
        entries: &ObjectValue,
        enumerable_only: bool,
    ) -> Option<Vec<String>> {
        if !Self::is_named_node_map_object(entries) {
            return None;
        }
        let node = Self::named_node_map_owner_node(entries)
            .filter(|node| self.dom.element(*node).is_some())?;
        let attrs = self.named_node_map_entries(node);
        let mut integer_keys = attrs
            .iter()
            .enumerate()
            .map(|(index, _)| (index as u64, index.to_string()))
            .collect::<Vec<_>>();
        let property_keys = if enumerable_only {
            Self::ordered_enumerable_string_keys(entries)
        } else {
            Self::ordered_visible_string_keys(entries)
        };
        for key in &property_keys {
            if let Some(index) = Self::own_property_integer_key(key)
                && !integer_keys.iter().any(|(existing, _)| *existing == index)
            {
                integer_keys.push((index, key.clone()));
            }
        }
        integer_keys.sort_by_key(|(index, _)| *index);
        let mut out = integer_keys
            .into_iter()
            .map(|(_, key)| key)
            .collect::<Vec<_>>();
        if !enumerable_only {
            out.push("length".to_string());
        }
        out.extend(attrs.iter().map(|(name, _)| name.clone()).filter(|key| {
            !property_keys.iter().any(|existing| existing == key)
                && self.named_node_map_named_property_is_visible(entries, key)
        }));
        out.extend(property_keys.into_iter().filter(|key| {
            Self::own_property_integer_key(key).is_none() && (enumerable_only || key != "length")
        }));
        Some(out)
    }

    pub(crate) fn named_node_map_synthesized_descriptor_value(
        &mut self,
        entries: &ObjectValue,
        key: &str,
    ) -> Option<Value> {
        if !Self::is_named_node_map_object(entries)
            || Self::own_property_descriptor_object_from_entries(entries, key).is_some()
        {
            return None;
        }
        let node = Self::named_node_map_owner_node(entries)
            .filter(|node| self.dom.element(*node).is_some())?;
        let attrs = self.named_node_map_entries(node);
        if key == "length" {
            return Some(Self::own_data_property_descriptor_with_attrs(
                Value::Number(attrs.len() as i64),
                true,
                false,
                true,
            ));
        }
        if let Some(index) = Self::own_property_integer_key(key) {
            let (name, value) = attrs.get(index as usize)?;
            return Some(Self::own_data_property_descriptor_with_attrs(
                Self::new_attr_object_value(name, value, Some(node)),
                true,
                true,
                true,
            ));
        }
        if !self.named_node_map_named_property_is_visible(entries, key) {
            return None;
        }
        let (name, value) = attrs.iter().find(|(name, _)| name == key)?;
        Some(Self::own_data_property_descriptor_with_attrs(
            Self::new_attr_object_value(name, value, Some(node)),
            true,
            true,
            true,
        ))
    }

    pub(crate) fn node_list_synthesized_keys(
        &mut self,
        nodes: &Rc<RefCell<NodeListValue>>,
        enumerable_only: bool,
    ) -> Vec<String> {
        let snapshot = self.node_list_snapshot(nodes);
        let mut integer_keys = snapshot
            .iter()
            .enumerate()
            .map(|(index, _)| (index as u64, index.to_string()))
            .collect::<Vec<_>>();
        let property_keys = {
            let nodes_ref = nodes.borrow();
            if enumerable_only {
                Self::ordered_enumerable_string_keys(&nodes_ref.properties)
            } else {
                Self::ordered_visible_string_keys(&nodes_ref.properties)
            }
        };
        for key in &property_keys {
            if let Some(index) = Self::own_property_integer_key(key)
                && !integer_keys.iter().any(|(existing, _)| *existing == index)
            {
                integer_keys.push((index, key.clone()));
            }
        }
        integer_keys.sort_by_key(|(index, _)| *index);
        let mut out = integer_keys
            .into_iter()
            .map(|(_, key)| key)
            .collect::<Vec<_>>();
        let named_keys = self
            .html_collection_named_entries(nodes)
            .into_iter()
            .map(|(name, _)| name)
            .filter(|key| {
                !property_keys.iter().any(|existing| existing == key)
                    && self.html_collection_named_property_is_visible(nodes, key)
            })
            .collect::<Vec<_>>();
        if !enumerable_only {
            out.push("length".to_string());
        }
        out.extend(named_keys);
        out.extend(property_keys.into_iter().filter(|key| {
            Self::own_property_integer_key(key).is_none() && (enumerable_only || key != "length")
        }));
        out
    }

    pub(crate) fn node_list_synthesized_descriptor_value(
        &mut self,
        nodes: &Rc<RefCell<NodeListValue>>,
        key: &str,
    ) -> Option<Value> {
        {
            let nodes_ref = nodes.borrow();
            if Self::own_property_descriptor_object_from_entries(&nodes_ref.properties, key)
                .is_some()
            {
                return None;
            }
        }
        let snapshot = self.node_list_snapshot(nodes);
        if key == "length" {
            return Some(Self::own_data_property_descriptor_with_attrs(
                Value::Number(snapshot.len() as i64),
                false,
                false,
                true,
            ));
        }
        if let Some(index) = Self::own_property_integer_key(key) {
            if let Some(node) = snapshot.get(index as usize).copied() {
                return Some(Self::own_data_property_descriptor_with_attrs(
                    self.node_list_item_value(nodes, node),
                    false,
                    true,
                    true,
                ));
            }
        }
        self.html_collection_named_property_value(nodes, key)
            .map(|value| Self::own_data_property_descriptor_with_attrs(value, false, true, true))
    }
}
