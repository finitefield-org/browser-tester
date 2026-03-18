use super::*;

impl Harness {
    pub(crate) fn data_attr_name_to_dataset_key(attr_name: &str) -> Option<String> {
        let raw = attr_name.strip_prefix("data-")?;
        if raw.is_empty() {
            return None;
        }
        let normalized = raw.to_ascii_lowercase();
        let chars = normalized.chars().collect::<Vec<_>>();
        let mut out = String::with_capacity(chars.len());
        let mut index = 0usize;
        while index < chars.len() {
            let ch = chars[index];
            if ch == '-' {
                if let Some(next) = chars.get(index + 1).copied() {
                    if next.is_ascii_lowercase() {
                        out.push(next.to_ascii_uppercase());
                        index += 2;
                        continue;
                    }
                }
                out.push(ch);
            } else {
                out.push(ch);
            }
            index += 1;
        }
        if out.is_empty() { None } else { Some(out) }
    }

    pub(crate) fn dataset_entries_for_node(&self, node: NodeId) -> Vec<(String, Value)> {
        let Some(element) = self.dom.element(node) else {
            return Vec::new();
        };
        let mut entries = element
            .attrs
            .iter()
            .filter_map(|(attr_name, attr_value)| {
                Self::data_attr_name_to_dataset_key(attr_name)
                    .map(|key| (key, Value::String(attr_value.clone())))
            })
            .collect::<Vec<_>>();
        entries.sort_by(|(left, _), (right, _)| left.cmp(right));
        entries
    }

    pub(crate) fn is_to_string_tag_property_key(&self, key: &str) -> bool {
        Self::symbol_id_from_storage_key(key)
            .and_then(|symbol_id| self.symbol_runtime.symbols_by_id.get(&symbol_id))
            .and_then(|symbol| symbol.description.as_deref())
            .is_some_and(|description| description == "Symbol.toStringTag")
            || key == "Symbol.toStringTag"
    }

    pub(crate) fn is_iterator_property_key(&self, key: &str) -> bool {
        Self::symbol_id_from_storage_key(key)
            .and_then(|symbol_id| self.symbol_runtime.symbols_by_id.get(&symbol_id))
            .and_then(|symbol| symbol.description.as_deref())
            .is_some_and(|description| description == "Symbol.iterator")
            || key == "Symbol.iterator"
    }

    pub(crate) fn is_string_method_name(name: &str) -> bool {
        matches!(
            name,
            "concat"
                | "endsWith"
                | "includes"
                | "normalize"
                | "slice"
                | "split"
                | "startsWith"
                | "substring"
        )
    }

    pub(crate) fn is_array_method_name(name: &str) -> bool {
        matches!(
            name,
            "forEach"
                | "map"
                | "flat"
                | "flatMap"
                | "filter"
                | "reduce"
                | "find"
                | "findIndex"
                | "some"
                | "every"
                | "values"
                | "keys"
                | "entries"
                | "fill"
                | "includes"
                | "indexOf"
                | "lastIndexOf"
                | "slice"
                | "join"
                | "concat"
                | "add"
                | "remove"
                | "clear"
                | "push"
                | "pop"
                | "shift"
                | "unshift"
                | "splice"
                | "sort"
                | "reverse"
        )
    }

    pub(crate) fn is_class_list_method_name(name: &str) -> bool {
        matches!(
            name,
            "add"
                | "remove"
                | "toggle"
                | "contains"
                | "replace"
                | "item"
                | "forEach"
                | "keys"
                | "values"
                | "entries"
                | "toString"
        )
    }

    pub(crate) fn is_named_node_map_method_name(name: &str) -> bool {
        matches!(
            name,
            "item"
                | "getNamedItem"
                | "setNamedItem"
                | "removeNamedItem"
                | "getNamedItemNS"
                | "setNamedItemNS"
                | "removeNamedItemNS"
                | "forEach"
                | "keys"
                | "values"
                | "entries"
        )
    }

    pub(crate) fn is_typed_array_method_name(name: &str) -> bool {
        matches!(
            name,
            "at" | "copyWithin"
                | "entries"
                | "join"
                | "keys"
                | "slice"
                | "subarray"
                | "values"
                | "with"
        )
    }
}
