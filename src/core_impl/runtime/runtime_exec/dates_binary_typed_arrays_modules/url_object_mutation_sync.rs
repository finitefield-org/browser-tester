use super::*;

impl Harness {
    pub(crate) fn sync_url_search_params_owner(&mut self, object: &Rc<RefCell<ObjectValue>>) {
        let (owner_id, pairs) = {
            let entries = object.borrow();
            let owner_id =
                match Self::object_get_entry(&entries, INTERNAL_URL_SEARCH_PARAMS_OWNER_ID_KEY) {
                    Some(Value::Number(id)) if id > 0 => usize::try_from(id).ok(),
                    _ => None,
                };
            let pairs = Self::url_search_params_pairs_from_object_entries(&entries);
            (owner_id, pairs)
        };
        let Some(owner_id) = owner_id else {
            return;
        };
        let Some(url_object) = self.browser_apis.url_objects.get(&owner_id).cloned() else {
            return;
        };

        let current_href = {
            let entries = url_object.borrow();
            Self::object_get_entry(&entries, "href")
                .map(|value| value.as_string())
                .unwrap_or_default()
        };
        let Some(mut parts) = LocationParts::parse(&current_href) else {
            return;
        };

        let serialized = serialize_url_search_params_pairs(&pairs);
        parts.search = if serialized.is_empty() {
            String::new()
        } else {
            format!("?{serialized}")
        };
        Self::normalize_url_parts_for_serialization(&mut parts);
        self.sync_url_object_entries_from_parts(&mut url_object.borrow_mut(), &parts);
    }
}
