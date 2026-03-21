use std::collections::BTreeMap;

use bt_script::{
    ElementHandle, HostBindings, HtmlCollectionScope, HtmlCollectionTarget, ListenerTarget,
    NodeHandle, ScriptFunction, ScriptRuntime,
};

#[derive(Default)]
struct NoopHost {
    microtasks: usize,
}

impl HostBindings for NoopHost {
    fn on_microtask_checkpoint(&mut self) -> bt_script::Result<()> {
        self.microtasks += 1;
        Ok(())
    }
}

fn origin_from_url(url: &str) -> String {
    let Some((scheme, rest)) = url.split_once(':') else {
        return "null".to_string();
    };

    let scheme = scheme.to_ascii_lowercase();
    let Some(after_slashes) = rest.strip_prefix("//") else {
        return "null".to_string();
    };

    let authority_end = after_slashes
        .find(['/', '?', '#'])
        .unwrap_or(after_slashes.len());
    let authority = &after_slashes[..authority_end];
    if authority.is_empty() {
        return "null".to_string();
    }

    format!("{scheme}://{authority}")
}

#[derive(Default)]
struct RecordingHost {
    elements: BTreeMap<String, ElementHandle>,
    text_content: BTreeMap<ElementHandle, String>,
    inner_html: BTreeMap<ElementHandle, String>,
    values: BTreeMap<ElementHandle, String>,
    checked: BTreeMap<ElementHandle, bool>,
    attributes: BTreeMap<(ElementHandle, String), String>,
    element_children_results: BTreeMap<ElementHandle, Vec<ElementHandle>>,
    element_tag_name_results: BTreeMap<ElementHandle, String>,
    element_labels_results: BTreeMap<ElementHandle, Vec<ElementHandle>>,
    html_collection_tag_name_items_results: BTreeMap<HtmlCollectionTarget, Vec<ElementHandle>>,
    html_collection_tag_name_ns_items_results: BTreeMap<HtmlCollectionTarget, Vec<ElementHandle>>,
    html_collection_class_name_items_results: BTreeMap<HtmlCollectionTarget, Vec<ElementHandle>>,
    html_collection_form_elements_items_results: BTreeMap<ElementHandle, Vec<ElementHandle>>,
    html_collection_select_options_items_results: BTreeMap<ElementHandle, Vec<ElementHandle>>,
    html_collection_select_selected_options_items_results:
        BTreeMap<ElementHandle, Vec<ElementHandle>>,
    html_collection_map_areas_items_results: BTreeMap<ElementHandle, Vec<ElementHandle>>,
    html_collection_table_bodies_items_results: BTreeMap<ElementHandle, Vec<ElementHandle>>,
    document_links_items_results: Vec<ElementHandle>,
    document_anchors_items_results: Vec<ElementHandle>,
    document_style_sheets_items_results: Vec<ElementHandle>,
    document_children_items_results: Vec<ElementHandle>,
    node_child_nodes_items_results: BTreeMap<HtmlCollectionScope, Vec<NodeHandle>>,
    node_text_content_results: BTreeMap<NodeHandle, String>,
    node_type_results: BTreeMap<NodeHandle, u8>,
    node_name_results: BTreeMap<NodeHandle, String>,
    table_rows_items_results: BTreeMap<ElementHandle, Vec<ElementHandle>>,
    row_cells_items_results: BTreeMap<ElementHandle, Vec<ElementHandle>>,
    html_collection_named_item_results: BTreeMap<(ElementHandle, String), Option<ElementHandle>>,
    html_collection_tag_name_named_item_results:
        BTreeMap<(HtmlCollectionTarget, String), Option<ElementHandle>>,
    html_collection_tag_name_ns_named_item_results:
        BTreeMap<(HtmlCollectionTarget, String), Option<ElementHandle>>,
    html_collection_class_name_named_item_results:
        BTreeMap<(HtmlCollectionTarget, String), Option<ElementHandle>>,
    html_collection_form_elements_named_item_results:
        BTreeMap<(ElementHandle, String), Option<ElementHandle>>,
    html_collection_form_elements_named_items_results:
        BTreeMap<(ElementHandle, String), Vec<ElementHandle>>,
    html_collection_select_options_named_item_results:
        BTreeMap<(ElementHandle, String), Option<ElementHandle>>,
    html_collection_select_selected_options_named_item_results:
        BTreeMap<(ElementHandle, String), Option<ElementHandle>>,
    html_collection_map_areas_named_item_results:
        BTreeMap<(ElementHandle, String), Option<ElementHandle>>,
    html_collection_table_bodies_named_item_results:
        BTreeMap<(ElementHandle, String), Option<ElementHandle>>,
    document_links_named_item_results: BTreeMap<String, Option<ElementHandle>>,
    document_anchors_named_item_results: BTreeMap<String, Option<ElementHandle>>,
    document_children_named_item_results: BTreeMap<String, Option<ElementHandle>>,
    table_rows_named_item_results: BTreeMap<(ElementHandle, String), Option<ElementHandle>>,
    row_cells_named_item_results: BTreeMap<(ElementHandle, String), Option<ElementHandle>>,
    document_query_selector_results: BTreeMap<String, Option<ElementHandle>>,
    document_query_selector_all_results: BTreeMap<String, Vec<ElementHandle>>,
    document_get_elements_by_name_results: BTreeMap<String, Vec<ElementHandle>>,
    document_document_element_result: Option<ElementHandle>,
    document_head_result: Option<ElementHandle>,
    document_body_result: Option<ElementHandle>,
    document_title_result: String,
    document_location_result: String,
    document_base_uri_calls: usize,
    document_origin_calls: usize,
    element_base_uri_calls: Vec<ElementHandle>,
    element_origin_calls: Vec<ElementHandle>,
    element_query_selector_results: BTreeMap<(ElementHandle, String), Option<ElementHandle>>,
    element_query_selector_all_results: BTreeMap<(ElementHandle, String), Vec<ElementHandle>>,
    element_closest_results: BTreeMap<(ElementHandle, String), Option<ElementHandle>>,
    element_children_calls: Vec<ElementHandle>,
    element_tag_name_calls: Vec<ElementHandle>,
    element_inner_html_calls: Vec<ElementHandle>,
    element_set_inner_html_calls: Vec<(ElementHandle, String)>,
    element_insert_adjacent_html_calls: Vec<(ElementHandle, String, String)>,
    element_labels_calls: Vec<ElementHandle>,
    html_collection_tag_name_items_calls: Vec<HtmlCollectionTarget>,
    html_collection_tag_name_ns_items_calls: Vec<HtmlCollectionTarget>,
    html_collection_class_name_items_calls: Vec<HtmlCollectionTarget>,
    html_collection_form_elements_items_calls: Vec<ElementHandle>,
    html_collection_select_options_items_calls: Vec<ElementHandle>,
    html_collection_select_selected_options_items_calls: Vec<ElementHandle>,
    html_collection_map_areas_items_calls: Vec<ElementHandle>,
    html_collection_table_bodies_items_calls: Vec<ElementHandle>,
    document_links_items_calls: usize,
    document_anchors_items_calls: usize,
    document_style_sheets_items_calls: usize,
    document_children_items_calls: usize,
    node_child_nodes_items_calls: Vec<HtmlCollectionScope>,
    node_text_content_calls: Vec<NodeHandle>,
    node_type_calls: Vec<NodeHandle>,
    node_name_calls: Vec<NodeHandle>,
    table_rows_items_calls: Vec<ElementHandle>,
    row_cells_items_calls: Vec<ElementHandle>,
    html_collection_named_item_calls: Vec<(ElementHandle, String)>,
    html_collection_tag_name_named_item_calls: Vec<(HtmlCollectionTarget, String)>,
    html_collection_tag_name_ns_named_item_calls: Vec<(HtmlCollectionTarget, String)>,
    html_collection_class_name_named_item_calls: Vec<(HtmlCollectionTarget, String)>,
    html_collection_form_elements_named_item_calls: Vec<(ElementHandle, String)>,
    html_collection_form_elements_named_items_calls: Vec<(ElementHandle, String)>,
    html_collection_select_options_named_item_calls: Vec<(ElementHandle, String)>,
    html_collection_select_selected_options_named_item_calls: Vec<(ElementHandle, String)>,
    html_collection_map_areas_named_item_calls: Vec<(ElementHandle, String)>,
    html_collection_table_bodies_named_item_calls: Vec<(ElementHandle, String)>,
    document_links_named_item_calls: Vec<String>,
    document_anchors_named_item_calls: Vec<String>,
    document_children_named_item_calls: Vec<String>,
    table_rows_named_item_calls: Vec<(ElementHandle, String)>,
    row_cells_named_item_calls: Vec<(ElementHandle, String)>,
    document_query_selector_calls: Vec<String>,
    document_query_selector_all_calls: Vec<String>,
    document_get_elements_by_name_calls: Vec<String>,
    document_document_element_calls: usize,
    document_head_calls: usize,
    document_body_calls: usize,
    document_title_calls: usize,
    document_set_title_calls: Vec<String>,
    document_location_calls: usize,
    document_set_location_calls: Vec<String>,
    element_query_selector_calls: Vec<(ElementHandle, String)>,
    element_query_selector_all_calls: Vec<(ElementHandle, String)>,
    element_closest_calls: Vec<(ElementHandle, String)>,
    element_matches_results: BTreeMap<(ElementHandle, String), bool>,
    element_matches_calls: Vec<(ElementHandle, String)>,
    listeners: Vec<(ListenerTarget, String, bool, ScriptFunction)>,
}

impl RecordingHost {
    fn seed_element(
        &mut self,
        id: impl Into<String>,
        handle: ElementHandle,
        text: impl Into<String>,
    ) {
        let id = id.into();
        self.elements.insert(id, handle);
        self.text_content.insert(handle, text.into());
    }

    fn seed_value(&mut self, handle: ElementHandle, value: impl Into<String>) {
        self.values.insert(handle, value.into());
    }

    fn seed_checked(&mut self, handle: ElementHandle, checked: bool) {
        self.checked.insert(handle, checked);
    }

    fn seed_attribute(
        &mut self,
        handle: ElementHandle,
        name: impl Into<String>,
        value: impl Into<String>,
    ) {
        self.attributes.insert((handle, name.into()), value.into());
    }

    fn seed_element_children(&mut self, element: ElementHandle, result: Vec<ElementHandle>) {
        self.element_children_results.insert(element, result);
    }

    fn seed_element_tag_name(&mut self, element: ElementHandle, tag_name: impl Into<String>) {
        self.element_tag_name_results
            .insert(element, tag_name.into());
    }

    fn seed_element_inner_html(&mut self, element: ElementHandle, html: impl Into<String>) {
        self.inner_html.insert(element, html.into());
    }

    fn seed_element_labels(&mut self, element: ElementHandle, result: Vec<ElementHandle>) {
        self.element_labels_results.insert(element, result);
    }

    fn seed_html_collection_tag_name_items(
        &mut self,
        collection: HtmlCollectionTarget,
        result: Vec<ElementHandle>,
    ) {
        self.html_collection_tag_name_items_results
            .insert(collection, result);
    }

    fn seed_html_collection_tag_name_ns_items(
        &mut self,
        collection: HtmlCollectionTarget,
        result: Vec<ElementHandle>,
    ) {
        self.html_collection_tag_name_ns_items_results
            .insert(collection, result);
    }

    fn seed_html_collection_class_name_items(
        &mut self,
        collection: HtmlCollectionTarget,
        result: Vec<ElementHandle>,
    ) {
        self.html_collection_class_name_items_results
            .insert(collection, result);
    }

    fn seed_html_collection_form_elements_items(
        &mut self,
        element: ElementHandle,
        result: Vec<ElementHandle>,
    ) {
        self.html_collection_form_elements_items_results
            .insert(element, result);
    }

    fn seed_html_collection_select_options_items(
        &mut self,
        element: ElementHandle,
        result: Vec<ElementHandle>,
    ) {
        self.html_collection_select_options_items_results
            .insert(element, result);
    }

    fn seed_html_collection_select_selected_options_items(
        &mut self,
        element: ElementHandle,
        result: Vec<ElementHandle>,
    ) {
        self.html_collection_select_selected_options_items_results
            .insert(element, result);
    }

    fn seed_html_collection_map_areas_items(
        &mut self,
        element: ElementHandle,
        result: Vec<ElementHandle>,
    ) {
        self.html_collection_map_areas_items_results
            .insert(element, result);
    }

    fn seed_html_collection_table_bodies_items(
        &mut self,
        element: ElementHandle,
        result: Vec<ElementHandle>,
    ) {
        self.html_collection_table_bodies_items_results
            .insert(element, result);
    }

    fn seed_document_links_items(&mut self, result: Vec<ElementHandle>) {
        self.document_links_items_results = result;
    }

    fn seed_document_anchors_items(&mut self, result: Vec<ElementHandle>) {
        self.document_anchors_items_results = result;
    }

    fn seed_document_style_sheets_items(&mut self, result: Vec<ElementHandle>) {
        self.document_style_sheets_items_results = result;
    }

    fn seed_document_children_items(&mut self, result: Vec<ElementHandle>) {
        self.document_children_items_results = result;
    }

    fn seed_node_child_nodes_items(&mut self, scope: HtmlCollectionScope, result: Vec<NodeHandle>) {
        self.node_child_nodes_items_results.insert(scope, result);
    }

    fn seed_node_text_content(&mut self, node: NodeHandle, result: impl Into<String>) {
        self.node_text_content_results.insert(node, result.into());
    }

    fn seed_node_type(&mut self, node: NodeHandle, result: u8) {
        self.node_type_results.insert(node, result);
    }

    fn seed_node_name(&mut self, node: NodeHandle, result: impl Into<String>) {
        self.node_name_results.insert(node, result.into());
    }

    fn seed_table_rows_items(&mut self, element: ElementHandle, result: Vec<ElementHandle>) {
        self.table_rows_items_results.insert(element, result);
    }

    fn seed_row_cells_items(&mut self, element: ElementHandle, result: Vec<ElementHandle>) {
        self.row_cells_items_results.insert(element, result);
    }

    fn seed_html_collection_named_item(
        &mut self,
        element: ElementHandle,
        name: impl Into<String>,
        result: Option<ElementHandle>,
    ) {
        self.html_collection_named_item_results
            .insert((element, name.into()), result);
    }

    fn seed_html_collection_tag_name_named_item(
        &mut self,
        collection: HtmlCollectionTarget,
        name: impl Into<String>,
        result: Option<ElementHandle>,
    ) {
        self.html_collection_tag_name_named_item_results
            .insert((collection, name.into()), result);
    }

    fn seed_html_collection_tag_name_ns_named_item(
        &mut self,
        collection: HtmlCollectionTarget,
        name: impl Into<String>,
        result: Option<ElementHandle>,
    ) {
        self.html_collection_tag_name_ns_named_item_results
            .insert((collection, name.into()), result);
    }

    fn seed_html_collection_class_name_named_item(
        &mut self,
        collection: HtmlCollectionTarget,
        name: impl Into<String>,
        result: Option<ElementHandle>,
    ) {
        self.html_collection_class_name_named_item_results
            .insert((collection, name.into()), result);
    }

    fn seed_html_collection_form_elements_named_item(
        &mut self,
        element: ElementHandle,
        name: impl Into<String>,
        result: Option<ElementHandle>,
    ) {
        self.html_collection_form_elements_named_item_results
            .insert((element, name.into()), result);
    }

    fn seed_html_collection_form_elements_named_items(
        &mut self,
        element: ElementHandle,
        name: impl Into<String>,
        result: Vec<ElementHandle>,
    ) {
        self.html_collection_form_elements_named_items_results
            .insert((element, name.into()), result);
    }

    fn seed_html_collection_select_options_named_item(
        &mut self,
        element: ElementHandle,
        name: impl Into<String>,
        result: Option<ElementHandle>,
    ) {
        self.html_collection_select_options_named_item_results
            .insert((element, name.into()), result);
    }

    fn seed_html_collection_select_selected_options_named_item(
        &mut self,
        element: ElementHandle,
        name: impl Into<String>,
        result: Option<ElementHandle>,
    ) {
        self.html_collection_select_selected_options_named_item_results
            .insert((element, name.into()), result);
    }

    fn seed_html_collection_map_areas_named_item(
        &mut self,
        element: ElementHandle,
        name: impl Into<String>,
        result: Option<ElementHandle>,
    ) {
        self.html_collection_map_areas_named_item_results
            .insert((element, name.into()), result);
    }

    fn seed_html_collection_table_bodies_named_item(
        &mut self,
        element: ElementHandle,
        name: impl Into<String>,
        result: Option<ElementHandle>,
    ) {
        self.html_collection_table_bodies_named_item_results
            .insert((element, name.into()), result);
    }

    fn seed_document_links_named_item(
        &mut self,
        name: impl Into<String>,
        result: Option<ElementHandle>,
    ) {
        self.document_links_named_item_results
            .insert(name.into(), result);
    }

    fn seed_document_anchors_named_item(
        &mut self,
        name: impl Into<String>,
        result: Option<ElementHandle>,
    ) {
        self.document_anchors_named_item_results
            .insert(name.into(), result);
    }

    fn seed_document_children_named_item(
        &mut self,
        name: impl Into<String>,
        result: Option<ElementHandle>,
    ) {
        self.document_children_named_item_results
            .insert(name.into(), result);
    }

    fn seed_table_rows_named_item(
        &mut self,
        element: ElementHandle,
        name: impl Into<String>,
        result: Option<ElementHandle>,
    ) {
        self.table_rows_named_item_results
            .insert((element, name.into()), result);
    }

    fn seed_row_cells_named_item(
        &mut self,
        element: ElementHandle,
        name: impl Into<String>,
        result: Option<ElementHandle>,
    ) {
        self.row_cells_named_item_results
            .insert((element, name.into()), result);
    }

    fn seed_document_query_selector(
        &mut self,
        selector: impl Into<String>,
        result: Option<ElementHandle>,
    ) {
        self.document_query_selector_results
            .insert(selector.into(), result);
    }

    fn seed_document_document_element(&mut self, result: Option<ElementHandle>) {
        self.document_document_element_result = result;
    }

    fn seed_document_head(&mut self, result: Option<ElementHandle>) {
        self.document_head_result = result;
    }

    fn seed_document_body(&mut self, result: Option<ElementHandle>) {
        self.document_body_result = result;
    }

    fn seed_document_title(&mut self, result: impl Into<String>) {
        self.document_title_result = result.into();
    }

    fn seed_document_location(&mut self, result: impl Into<String>) {
        self.document_location_result = result.into();
    }

    fn seed_document_query_selector_all(
        &mut self,
        selector: impl Into<String>,
        result: Vec<ElementHandle>,
    ) {
        self.document_query_selector_all_results
            .insert(selector.into(), result);
    }

    fn seed_document_get_elements_by_name(
        &mut self,
        name: impl Into<String>,
        result: Vec<ElementHandle>,
    ) {
        self.document_get_elements_by_name_results
            .insert(name.into(), result);
    }

    fn seed_element_query_selector(
        &mut self,
        element: ElementHandle,
        selector: impl Into<String>,
        result: Option<ElementHandle>,
    ) {
        self.element_query_selector_results
            .insert((element, selector.into()), result);
    }

    fn seed_element_query_selector_all(
        &mut self,
        element: ElementHandle,
        selector: impl Into<String>,
        result: Vec<ElementHandle>,
    ) {
        self.element_query_selector_all_results
            .insert((element, selector.into()), result);
    }

    fn seed_element_closest(
        &mut self,
        element: ElementHandle,
        selector: impl Into<String>,
        result: Option<ElementHandle>,
    ) {
        self.element_closest_results
            .insert((element, selector.into()), result);
    }

    fn seed_element_matches(
        &mut self,
        element: ElementHandle,
        selector: impl Into<String>,
        result: bool,
    ) {
        self.element_matches_results
            .insert((element, selector.into()), result);
    }
}

impl HostBindings for RecordingHost {
    fn document_get_element_by_id(&mut self, id: &str) -> bt_script::Result<Option<ElementHandle>> {
        Ok(self.elements.get(id).copied())
    }

    fn document_document_element(&mut self) -> bt_script::Result<Option<ElementHandle>> {
        self.document_document_element_calls += 1;
        Ok(self.document_document_element_result)
    }

    fn document_head(&mut self) -> bt_script::Result<Option<ElementHandle>> {
        self.document_head_calls += 1;
        Ok(self.document_head_result)
    }

    fn document_body(&mut self) -> bt_script::Result<Option<ElementHandle>> {
        self.document_body_calls += 1;
        Ok(self.document_body_result)
    }

    fn document_title(&mut self) -> bt_script::Result<String> {
        self.document_title_calls += 1;
        Ok(self.document_title_result.clone())
    }

    fn document_set_title(&mut self, value: &str) -> bt_script::Result<()> {
        self.document_set_title_calls.push(value.to_string());
        self.document_title_result = value.to_string();
        Ok(())
    }

    fn document_location(&mut self) -> bt_script::Result<String> {
        self.document_location_calls += 1;
        Ok(self.document_location_result.clone())
    }

    fn document_url(&mut self) -> bt_script::Result<String> {
        self.document_location_calls += 1;
        Ok(self.document_location_result.clone())
    }

    fn document_document_uri(&mut self) -> bt_script::Result<String> {
        self.document_location_calls += 1;
        Ok(self.document_location_result.clone())
    }

    fn document_base_uri(&mut self) -> bt_script::Result<String> {
        self.document_base_uri_calls += 1;
        Ok(self.document_location_result.clone())
    }

    fn document_origin(&mut self) -> bt_script::Result<String> {
        self.document_origin_calls += 1;
        Ok(origin_from_url(&self.document_location_result))
    }

    fn document_set_location(&mut self, value: &str) -> bt_script::Result<()> {
        self.document_set_location_calls.push(value.to_string());
        self.document_location_result = value.to_string();
        Ok(())
    }

    fn element_text_content(&mut self, element: ElementHandle) -> bt_script::Result<String> {
        Ok(self.text_content.get(&element).cloned().unwrap_or_default())
    }

    fn element_children(
        &mut self,
        element: ElementHandle,
    ) -> bt_script::Result<Vec<ElementHandle>> {
        self.element_children_calls.push(element);
        Ok(self
            .element_children_results
            .get(&element)
            .cloned()
            .unwrap_or_default())
    }

    fn element_tag_name(&mut self, element: ElementHandle) -> bt_script::Result<String> {
        self.element_tag_name_calls.push(element);
        Ok(self
            .element_tag_name_results
            .get(&element)
            .cloned()
            .unwrap_or_default())
    }

    fn element_base_uri(&mut self, element: ElementHandle) -> bt_script::Result<String> {
        self.element_base_uri_calls.push(element);
        Ok(self.document_location_result.clone())
    }

    fn element_origin(&mut self, element: ElementHandle) -> bt_script::Result<String> {
        self.element_origin_calls.push(element);
        Ok(origin_from_url(&self.document_location_result))
    }

    fn element_labels(&mut self, element: ElementHandle) -> bt_script::Result<Vec<ElementHandle>> {
        self.element_labels_calls.push(element);
        Ok(self
            .element_labels_results
            .get(&element)
            .cloned()
            .unwrap_or_default())
    }

    fn html_collection_tag_name_items(
        &mut self,
        collection: HtmlCollectionTarget,
    ) -> bt_script::Result<Vec<ElementHandle>> {
        self.html_collection_tag_name_items_calls
            .push(collection.clone());
        Ok(self
            .html_collection_tag_name_items_results
            .get(&collection)
            .cloned()
            .unwrap_or_default())
    }

    fn html_collection_tag_name_ns_items(
        &mut self,
        collection: HtmlCollectionTarget,
    ) -> bt_script::Result<Vec<ElementHandle>> {
        self.html_collection_tag_name_ns_items_calls
            .push(collection.clone());
        Ok(self
            .html_collection_tag_name_ns_items_results
            .get(&collection)
            .cloned()
            .unwrap_or_default())
    }

    fn html_collection_named_item(
        &mut self,
        element: ElementHandle,
        name: &str,
    ) -> bt_script::Result<Option<ElementHandle>> {
        self.html_collection_named_item_calls
            .push((element, name.to_string()));
        Ok(self
            .html_collection_named_item_results
            .get(&(element, name.to_string()))
            .copied()
            .flatten())
    }

    fn html_collection_tag_name_named_item(
        &mut self,
        collection: HtmlCollectionTarget,
        name: &str,
    ) -> bt_script::Result<Option<ElementHandle>> {
        self.html_collection_tag_name_named_item_calls
            .push((collection.clone(), name.to_string()));
        Ok(self
            .html_collection_tag_name_named_item_results
            .get(&(collection, name.to_string()))
            .copied()
            .flatten())
    }

    fn html_collection_tag_name_ns_named_item(
        &mut self,
        collection: HtmlCollectionTarget,
        name: &str,
    ) -> bt_script::Result<Option<ElementHandle>> {
        self.html_collection_tag_name_ns_named_item_calls
            .push((collection.clone(), name.to_string()));
        Ok(self
            .html_collection_tag_name_ns_named_item_results
            .get(&(collection, name.to_string()))
            .copied()
            .flatten())
    }

    fn html_collection_class_name_items(
        &mut self,
        collection: HtmlCollectionTarget,
    ) -> bt_script::Result<Vec<ElementHandle>> {
        self.html_collection_class_name_items_calls
            .push(collection.clone());
        Ok(self
            .html_collection_class_name_items_results
            .get(&collection)
            .cloned()
            .unwrap_or_default())
    }

    fn html_collection_class_name_named_item(
        &mut self,
        collection: HtmlCollectionTarget,
        name: &str,
    ) -> bt_script::Result<Option<ElementHandle>> {
        self.html_collection_class_name_named_item_calls
            .push((collection.clone(), name.to_string()));
        Ok(self
            .html_collection_class_name_named_item_results
            .get(&(collection, name.to_string()))
            .copied()
            .flatten())
    }

    fn html_collection_form_elements_items(
        &mut self,
        element: ElementHandle,
    ) -> bt_script::Result<Vec<ElementHandle>> {
        self.html_collection_form_elements_items_calls.push(element);
        Ok(self
            .html_collection_form_elements_items_results
            .get(&element)
            .cloned()
            .unwrap_or_default())
    }

    fn html_collection_form_elements_named_item(
        &mut self,
        element: ElementHandle,
        name: &str,
    ) -> bt_script::Result<Option<ElementHandle>> {
        self.html_collection_form_elements_named_item_calls
            .push((element, name.to_string()));
        Ok(self
            .html_collection_form_elements_named_item_results
            .get(&(element, name.to_string()))
            .copied()
            .flatten())
    }

    fn html_collection_form_elements_named_items(
        &mut self,
        element: ElementHandle,
        name: &str,
    ) -> bt_script::Result<Vec<ElementHandle>> {
        self.html_collection_form_elements_named_item_calls
            .push((element, name.to_string()));
        self.html_collection_form_elements_named_items_calls
            .push((element, name.to_string()));
        if let Some(result) = self
            .html_collection_form_elements_named_items_results
            .get(&(element, name.to_string()))
        {
            return Ok(result.clone());
        }

        Ok(self
            .html_collection_form_elements_named_item_results
            .get(&(element, name.to_string()))
            .copied()
            .flatten()
            .into_iter()
            .collect())
    }

    fn html_collection_select_options_items(
        &mut self,
        element: ElementHandle,
    ) -> bt_script::Result<Vec<ElementHandle>> {
        self.html_collection_select_options_items_calls
            .push(element);
        Ok(self
            .html_collection_select_options_items_results
            .get(&element)
            .cloned()
            .unwrap_or_default())
    }

    fn html_collection_select_options_named_item(
        &mut self,
        element: ElementHandle,
        name: &str,
    ) -> bt_script::Result<Option<ElementHandle>> {
        self.html_collection_select_options_named_item_calls
            .push((element, name.to_string()));
        Ok(self
            .html_collection_select_options_named_item_results
            .get(&(element, name.to_string()))
            .copied()
            .flatten())
    }

    fn html_collection_select_selected_options_items(
        &mut self,
        element: ElementHandle,
    ) -> bt_script::Result<Vec<ElementHandle>> {
        self.html_collection_select_selected_options_items_calls
            .push(element);
        Ok(self
            .html_collection_select_selected_options_items_results
            .get(&element)
            .cloned()
            .unwrap_or_default())
    }

    fn html_collection_select_selected_options_named_item(
        &mut self,
        element: ElementHandle,
        name: &str,
    ) -> bt_script::Result<Option<ElementHandle>> {
        self.html_collection_select_selected_options_named_item_calls
            .push((element, name.to_string()));
        Ok(self
            .html_collection_select_selected_options_named_item_results
            .get(&(element, name.to_string()))
            .copied()
            .flatten())
    }

    fn html_collection_map_areas_items(
        &mut self,
        element: ElementHandle,
    ) -> bt_script::Result<Vec<ElementHandle>> {
        self.html_collection_map_areas_items_calls.push(element);
        Ok(self
            .html_collection_map_areas_items_results
            .get(&element)
            .cloned()
            .unwrap_or_default())
    }

    fn html_collection_map_areas_named_item(
        &mut self,
        element: ElementHandle,
        name: &str,
    ) -> bt_script::Result<Option<ElementHandle>> {
        self.html_collection_map_areas_named_item_calls
            .push((element, name.to_string()));
        Ok(self
            .html_collection_map_areas_named_item_results
            .get(&(element, name.to_string()))
            .copied()
            .flatten())
    }

    fn html_collection_table_bodies_items(
        &mut self,
        element: ElementHandle,
    ) -> bt_script::Result<Vec<ElementHandle>> {
        self.html_collection_table_bodies_items_calls.push(element);
        Ok(self
            .html_collection_table_bodies_items_results
            .get(&element)
            .cloned()
            .unwrap_or_default())
    }

    fn html_collection_table_bodies_named_item(
        &mut self,
        element: ElementHandle,
        name: &str,
    ) -> bt_script::Result<Option<ElementHandle>> {
        self.html_collection_table_bodies_named_item_calls
            .push((element, name.to_string()));
        Ok(self
            .html_collection_table_bodies_named_item_results
            .get(&(element, name.to_string()))
            .copied()
            .flatten())
    }

    fn html_collection_document_links_items(&mut self) -> bt_script::Result<Vec<ElementHandle>> {
        self.document_links_items_calls += 1;
        Ok(self.document_links_items_results.clone())
    }

    fn html_collection_document_links_named_item(
        &mut self,
        name: &str,
    ) -> bt_script::Result<Option<ElementHandle>> {
        self.document_links_named_item_calls.push(name.to_string());
        Ok(self
            .document_links_named_item_results
            .get(name)
            .copied()
            .flatten())
    }

    fn html_collection_document_anchors_items(&mut self) -> bt_script::Result<Vec<ElementHandle>> {
        self.document_anchors_items_calls += 1;
        Ok(self.document_anchors_items_results.clone())
    }

    fn html_collection_document_anchors_named_item(
        &mut self,
        name: &str,
    ) -> bt_script::Result<Option<ElementHandle>> {
        self.document_anchors_named_item_calls
            .push(name.to_string());
        Ok(self
            .document_anchors_named_item_results
            .get(name)
            .copied()
            .flatten())
    }

    fn html_collection_document_children_items(&mut self) -> bt_script::Result<Vec<ElementHandle>> {
        self.document_children_items_calls += 1;
        Ok(self.document_children_items_results.clone())
    }

    fn html_collection_document_children_named_item(
        &mut self,
        name: &str,
    ) -> bt_script::Result<Option<ElementHandle>> {
        self.document_children_named_item_calls
            .push(name.to_string());
        Ok(self
            .document_children_named_item_results
            .get(name)
            .copied()
            .flatten())
    }

    fn html_collection_table_rows_items(
        &mut self,
        element: ElementHandle,
    ) -> bt_script::Result<Vec<ElementHandle>> {
        self.table_rows_items_calls.push(element);
        Ok(self
            .table_rows_items_results
            .get(&element)
            .cloned()
            .unwrap_or_default())
    }

    fn html_collection_table_rows_named_item(
        &mut self,
        element: ElementHandle,
        name: &str,
    ) -> bt_script::Result<Option<ElementHandle>> {
        self.table_rows_named_item_calls
            .push((element, name.to_string()));
        Ok(self
            .table_rows_named_item_results
            .get(&(element, name.to_string()))
            .copied()
            .flatten())
    }

    fn html_collection_row_cells_items(
        &mut self,
        element: ElementHandle,
    ) -> bt_script::Result<Vec<ElementHandle>> {
        self.row_cells_items_calls.push(element);
        Ok(self
            .row_cells_items_results
            .get(&element)
            .cloned()
            .unwrap_or_default())
    }

    fn html_collection_row_cells_named_item(
        &mut self,
        element: ElementHandle,
        name: &str,
    ) -> bt_script::Result<Option<ElementHandle>> {
        self.row_cells_named_item_calls
            .push((element, name.to_string()));
        Ok(self
            .row_cells_named_item_results
            .get(&(element, name.to_string()))
            .copied()
            .flatten())
    }

    fn element_set_text_content(
        &mut self,
        element: ElementHandle,
        value: &str,
    ) -> bt_script::Result<()> {
        self.text_content.insert(element, value.to_string());
        Ok(())
    }

    fn element_inner_html(&mut self, element: ElementHandle) -> bt_script::Result<String> {
        self.element_inner_html_calls.push(element);
        Ok(self.inner_html.get(&element).cloned().unwrap_or_default())
    }

    fn element_set_inner_html(
        &mut self,
        element: ElementHandle,
        value: &str,
    ) -> bt_script::Result<()> {
        self.element_set_inner_html_calls
            .push((element, value.to_string()));
        self.inner_html.insert(element, value.to_string());
        Ok(())
    }

    fn element_value(&mut self, element: ElementHandle) -> bt_script::Result<String> {
        Ok(self
            .values
            .get(&element)
            .cloned()
            .or_else(|| self.text_content.get(&element).cloned())
            .unwrap_or_default())
    }

    fn element_set_value(&mut self, element: ElementHandle, value: &str) -> bt_script::Result<()> {
        self.values.insert(element, value.to_string());
        Ok(())
    }

    fn element_checked(&mut self, element: ElementHandle) -> bt_script::Result<bool> {
        Ok(self.checked.get(&element).copied().unwrap_or(false))
    }

    fn element_set_checked(
        &mut self,
        element: ElementHandle,
        checked: bool,
    ) -> bt_script::Result<()> {
        self.checked.insert(element, checked);
        Ok(())
    }

    fn element_get_attribute(
        &mut self,
        element: ElementHandle,
        name: &str,
    ) -> bt_script::Result<Option<String>> {
        Ok(self.attributes.get(&(element, name.to_string())).cloned())
    }

    fn element_set_attribute(
        &mut self,
        element: ElementHandle,
        name: &str,
        value: &str,
    ) -> bt_script::Result<()> {
        self.attributes
            .insert((element, name.to_string()), value.to_string());
        Ok(())
    }

    fn element_remove_attribute(
        &mut self,
        element: ElementHandle,
        name: &str,
    ) -> bt_script::Result<()> {
        self.attributes.remove(&(element, name.to_string()));
        Ok(())
    }

    fn element_has_attribute(
        &mut self,
        element: ElementHandle,
        name: &str,
    ) -> bt_script::Result<bool> {
        Ok(self.attributes.contains_key(&(element, name.to_string())))
    }

    fn element_toggle_attribute(
        &mut self,
        element: ElementHandle,
        name: &str,
        force: Option<bool>,
    ) -> bt_script::Result<bool> {
        let key = (element, name.to_string());
        let has_attr = self.attributes.contains_key(&key);
        let now_present = match force {
            Some(true) => {
                if !has_attr {
                    self.attributes.insert(key.clone(), String::new());
                }
                true
            }
            Some(false) => {
                self.attributes.remove(&key);
                false
            }
            None => {
                if has_attr {
                    self.attributes.remove(&key);
                    false
                } else {
                    self.attributes.insert(key.clone(), String::new());
                    true
                }
            }
        };
        Ok(now_present)
    }

    fn document_query_selector(
        &mut self,
        selector: &str,
    ) -> bt_script::Result<Option<ElementHandle>> {
        self.document_query_selector_calls
            .push(selector.to_string());
        Ok(self
            .document_query_selector_results
            .get(selector)
            .copied()
            .flatten())
    }

    fn document_query_selector_all(
        &mut self,
        selector: &str,
    ) -> bt_script::Result<Vec<ElementHandle>> {
        self.document_query_selector_all_calls
            .push(selector.to_string());
        Ok(self
            .document_query_selector_all_results
            .get(selector)
            .cloned()
            .unwrap_or_default())
    }

    fn document_get_elements_by_name(
        &mut self,
        name: &str,
    ) -> bt_script::Result<Vec<ElementHandle>> {
        self.document_get_elements_by_name_calls
            .push(name.to_string());
        Ok(self
            .document_get_elements_by_name_results
            .get(name)
            .cloned()
            .unwrap_or_default())
    }

    fn document_style_sheets_items(&mut self) -> bt_script::Result<Vec<ElementHandle>> {
        self.document_style_sheets_items_calls += 1;
        Ok(self.document_style_sheets_items_results.clone())
    }

    fn node_child_nodes_items(
        &mut self,
        scope: HtmlCollectionScope,
    ) -> bt_script::Result<Vec<NodeHandle>> {
        self.node_child_nodes_items_calls.push(scope.clone());
        Ok(self
            .node_child_nodes_items_results
            .get(&scope)
            .cloned()
            .unwrap_or_default())
    }

    fn node_text_content(&mut self, node: NodeHandle) -> bt_script::Result<String> {
        self.node_text_content_calls.push(node);
        Ok(self
            .node_text_content_results
            .get(&node)
            .cloned()
            .unwrap_or_default())
    }

    fn node_type(&mut self, node: NodeHandle) -> bt_script::Result<u8> {
        self.node_type_calls.push(node);
        Ok(self.node_type_results.get(&node).copied().unwrap_or(0))
    }

    fn node_name(&mut self, node: NodeHandle) -> bt_script::Result<String> {
        self.node_name_calls.push(node);
        Ok(self
            .node_name_results
            .get(&node)
            .cloned()
            .unwrap_or_default())
    }

    fn element_query_selector(
        &mut self,
        element: ElementHandle,
        selector: &str,
    ) -> bt_script::Result<Option<ElementHandle>> {
        self.element_query_selector_calls
            .push((element, selector.to_string()));
        Ok(self
            .element_query_selector_results
            .get(&(element, selector.to_string()))
            .copied()
            .flatten())
    }

    fn element_query_selector_all(
        &mut self,
        element: ElementHandle,
        selector: &str,
    ) -> bt_script::Result<Vec<ElementHandle>> {
        self.element_query_selector_all_calls
            .push((element, selector.to_string()));
        Ok(self
            .element_query_selector_all_results
            .get(&(element, selector.to_string()))
            .cloned()
            .unwrap_or_default())
    }

    fn element_matches(
        &mut self,
        element: ElementHandle,
        selector: &str,
    ) -> bt_script::Result<bool> {
        self.element_matches_calls
            .push((element, selector.to_string()));
        Ok(self
            .element_matches_results
            .get(&(element, selector.to_string()))
            .copied()
            .unwrap_or(false))
    }

    fn element_closest(
        &mut self,
        element: ElementHandle,
        selector: &str,
    ) -> bt_script::Result<Option<ElementHandle>> {
        self.element_closest_calls
            .push((element, selector.to_string()));
        Ok(self
            .element_closest_results
            .get(&(element, selector.to_string()))
            .copied()
            .flatten())
    }

    fn element_insert_adjacent_html(
        &mut self,
        element: ElementHandle,
        position: &str,
        value: &str,
    ) -> bt_script::Result<()> {
        self.element_insert_adjacent_html_calls.push((
            element,
            position.to_string(),
            value.to_string(),
        ));
        Ok(())
    }

    fn register_event_listener_with_capture(
        &mut self,
        target: ListenerTarget,
        event_type: &str,
        capture: bool,
        handler: ScriptFunction,
    ) -> bt_script::Result<()> {
        self.listeners
            .push((target, event_type.to_string(), capture, handler));
        Ok(())
    }
}

#[test]
fn runtime_tracks_microtask_queue_depth() {
    let mut runtime = ScriptRuntime::new();
    let mut host = NoopHost::default();
    runtime.queue_microtask();
    runtime.queue_microtask();
    runtime
        .run_microtasks(&mut host)
        .expect("runtime should drain microtasks");
    assert_eq!(host.microtasks, 2);
    assert_eq!(runtime.queued_microtasks(), 0);
}

#[test]
fn runtime_mutates_dom_through_host_bindings() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(1), "");

    runtime
        .eval_program(
            "document.getElementById('out').textContent = 'Hello';",
            "inline-script",
            &mut host,
        )
        .expect("script should mutate text content");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(1))
            .map(String::as_str),
        Some("Hello")
    );
    assert!(host.listeners.is_empty());
}

#[test]
fn runtime_registers_event_handlers() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("run", ElementHandle::new(1), "");
    host.seed_element("out", ElementHandle::new(2), "");

    runtime
        .eval_program(
            "document.getElementById('run').addEventListener('click', () => { document.getElementById('out').textContent = 'clicked'; });",
            "inline-script",
            &mut host,
        )
        .expect("script should register listeners");

    assert_eq!(host.listeners.len(), 1);
    assert_eq!(
        host.listeners[0].0,
        ListenerTarget::Element(ElementHandle::new(1))
    );
    assert_eq!(host.listeners[0].1, "click");
    assert!(!host.listeners[0].2);
    assert!(
        host.listeners[0]
            .3
            .body_source
            .contains("textContent = 'clicked'")
    );
}

#[test]
fn runtime_registers_capturing_event_handlers() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("run", ElementHandle::new(1), "");

    runtime
        .eval_program(
            "document.getElementById('run').addEventListener('click', (event) => { event.preventDefault(); }, true);",
            "inline-script",
            &mut host,
        )
        .expect("script should register listeners");

    assert_eq!(host.listeners.len(), 1);
    assert_eq!(
        host.listeners[0].0,
        ListenerTarget::Element(ElementHandle::new(1))
    );
    assert_eq!(host.listeners[0].1, "click");
    assert!(host.listeners[0].2);
    assert_eq!(host.listeners[0].3.params, vec!["event".to_string()]);
}

#[test]
fn runtime_reads_and_writes_form_control_state() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("name", ElementHandle::new(1), "");
    host.seed_element("agree", ElementHandle::new(2), "");
    host.seed_element("out", ElementHandle::new(3), "");
    host.seed_value(ElementHandle::new(1), "");
    host.seed_checked(ElementHandle::new(2), false);

    runtime
        .eval_program(
            "document.getElementById('name').value = 'Alice'; document.getElementById('agree').checked = true; document.getElementById('out').textContent = document.getElementById('name').value + ':' + String(document.getElementById('agree').checked);",
            "inline-script",
            &mut host,
        )
        .expect("script should mutate form controls");

    assert_eq!(
        host.values.get(&ElementHandle::new(1)).map(String::as_str),
        Some("Alice")
    );
    assert_eq!(
        host.checked.get(&ElementHandle::new(2)).copied(),
        Some(true)
    );
    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(3))
            .map(String::as_str),
        Some("Alice:true")
    );
}

#[test]
fn runtime_supports_attribute_reflection_methods() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("root", ElementHandle::new(1), "");
    host.seed_attribute(ElementHandle::new(1), "data-flag", "");

    runtime
        .eval_program(
            "const root = document.getElementById('root'); const before = root.hasAttribute('data-flag'); const removed = root.toggleAttribute('data-flag'); const missing = root.hasAttribute('data-flag'); const forced = root.toggleAttribute('data-flag', true); root.setAttribute('data-label', 'Hello'); const label = root.getAttribute('data-label'); root.removeAttribute('data-label'); document.getElementById('root').textContent = String(before) + ':' + String(removed) + ':' + String(missing) + ':' + String(forced) + ':' + label + ':' + String(root.getAttribute('data-label'));",
            "inline-script",
            &mut host,
        )
        .expect("attribute reflection should dispatch through host bindings");

    assert_eq!(
        host.attributes
            .get(&(ElementHandle::new(1), "data-flag".to_string()))
            .map(String::as_str),
        Some("")
    );
    assert!(
        !host
            .attributes
            .contains_key(&(ElementHandle::new(1), "data-label".to_string()))
    );
    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(1))
            .map(String::as_str),
        Some("true:false:false:true:Hello:null")
    );
}

#[test]
fn runtime_supports_classname_classlist_and_dataset_views() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("root", ElementHandle::new(1), "");
    host.seed_element("out", ElementHandle::new(2), "");
    host.seed_attribute(ElementHandle::new(1), "class", "primary secondary");
    host.seed_attribute(ElementHandle::new(1), "data-kind", "App");

    runtime
        .eval_program(
            "const root = document.getElementById('root'); const before = root.classList.length; const contains = root.classList.contains('primary'); root.classList.add('tertiary'); root.classList.remove('secondary'); const toggled = root.classList.toggle('active'); root.dataset.userId = '42'; document.getElementById('out').textContent = root.className + ':' + String(before) + ':' + String(contains) + ':' + String(toggled) + ':' + root.dataset.kind + ':' + root.dataset.userId + ':' + String(root.classList) + ':' + String(root.dataset);",
            "inline-script",
            &mut host,
        )
        .expect("class and dataset views should dispatch through host bindings");

    assert_eq!(
        host.attributes
            .get(&(ElementHandle::new(1), "class".to_string()))
            .map(String::as_str),
        Some("primary tertiary active")
    );
    assert_eq!(
        host.attributes
            .get(&(ElementHandle::new(1), "data-kind".to_string()))
            .map(String::as_str),
        Some("App")
    );
    assert_eq!(
        host.attributes
            .get(&(ElementHandle::new(1), "data-user-id".to_string()))
            .map(String::as_str),
        Some("42")
    );
    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(2))
            .map(String::as_str),
        Some(
            "primary tertiary active:2:true:true:App:42:[object DOMTokenList]:[object DOMStringMap]"
        )
    );
}

#[test]
fn runtime_resolves_document_root_head_and_body_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("html", ElementHandle::new(1), "");
    host.seed_element("head", ElementHandle::new(2), "");
    host.seed_element("body", ElementHandle::new(3), "");
    host.seed_element("out", ElementHandle::new(4), "");
    host.seed_attribute(ElementHandle::new(1), "id", "html");
    host.seed_attribute(ElementHandle::new(2), "id", "head");
    host.seed_attribute(ElementHandle::new(3), "id", "body");
    host.seed_document_document_element(Some(ElementHandle::new(1)));
    host.seed_document_head(Some(ElementHandle::new(2)));
    host.seed_document_body(Some(ElementHandle::new(3)));

    runtime
        .eval_program(
            "const html = document.documentElement; const head = document.head; const body = document.body; document.getElementById('out').textContent = html.getAttribute('id') + ':' + head.getAttribute('id') + ':' + body.getAttribute('id');",
            "inline-script",
            &mut host,
        )
        .expect("document root/head/body should resolve through host bindings");

    assert_eq!(host.document_document_element_calls, 1);
    assert_eq!(host.document_head_calls, 1);
    assert_eq!(host.document_body_calls, 1);
    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(4))
            .map(String::as_str),
        Some("html:head:body")
    );
}

#[test]
fn runtime_resolves_document_title_getter_setter_and_window_alias() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(1), "");
    host.seed_document_title("Initial");

    runtime
        .eval_program(
            "const before = document.title; document.title = 'Updated'; const after = window.title; document.getElementById('out').textContent = before + ':' + after;",
            "inline-script",
            &mut host,
        )
        .expect("document.title should resolve through host bindings");

    assert_eq!(host.document_title_calls, 2);
    assert_eq!(host.document_set_title_calls, vec!["Updated".to_string()]);
    assert_eq!(host.document_title_result, "Updated");
    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(1))
            .map(String::as_str),
        Some("Initial:Updated")
    );
}

#[test]
fn runtime_resolves_document_location_getter_setter_and_window_alias() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(1), "");
    host.seed_document_location("https://example.test/start");

    runtime
        .eval_program(
            "const before = document.location; document.location = 'https://example.test/next'; const after = window.location; document.getElementById('out').textContent = before + ':' + after;",
            "inline-script",
            &mut host,
        )
        .expect("document.location should resolve through host bindings");

    assert_eq!(host.document_location_calls, 2);
    assert_eq!(
        host.document_set_location_calls,
        vec!["https://example.test/next".to_string()]
    );
    assert_eq!(host.document_location_result, "https://example.test/next");
    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(1))
            .map(String::as_str),
        Some("https://example.test/start:https://example.test/next")
    );
}

#[test]
fn runtime_resolves_document_url_and_document_uri_aliases() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(1), "");
    host.seed_document_location("https://example.test/start");

    runtime
        .eval_program(
            "const beforeLocation = document.location; const beforeUrl = document.URL; const beforeDocumentUri = document.documentURI; const beforeWindowLocation = window.location; document.getElementById('out').textContent = beforeLocation + ':' + beforeUrl + ':' + beforeDocumentUri + ':' + beforeWindowLocation;",
            "inline-script",
            &mut host,
        )
        .expect("document URL aliases should resolve through host bindings");

    assert_eq!(host.document_location_calls, 4);
    assert_eq!(host.document_location_result, "https://example.test/start");
    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(1))
            .map(String::as_str),
        Some(
            "https://example.test/start:https://example.test/start:https://example.test/start:https://example.test/start"
        )
    );
}

#[test]
fn runtime_resolves_document_base_uri_and_element_base_uri_aliases() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(1), "");
    host.seed_document_location("https://example.test/start");

    runtime
        .eval_program(
            "const out = document.getElementById('out'); const beforeDocumentBaseUri = document.baseURI; const beforeElementBaseUri = out.baseURI; out.textContent = beforeDocumentBaseUri + ':' + beforeElementBaseUri;",
            "inline-script",
            &mut host,
        )
        .expect("document.baseURI should resolve through host bindings");

    assert_eq!(host.document_base_uri_calls, 1);
    assert_eq!(host.element_base_uri_calls, vec![ElementHandle::new(1)]);
    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(1))
            .map(String::as_str),
        Some("https://example.test/start:https://example.test/start")
    );
}

#[test]
fn runtime_resolves_document_origin_and_element_origin_aliases() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("root", ElementHandle::new(1), "root");
    host.seed_element("child", ElementHandle::new(2), "child");
    host.seed_document_location("https://example.test:8443/start?x#y");

    runtime
        .eval_program(
            "const root = document.getElementById('root'); const child = document.getElementById('child'); document.getElementById('root').textContent = document.origin + ':' + window.origin + ':' + root.origin + ':' + child.origin;",
            "inline-script",
            &mut host,
        )
        .expect("document.origin should resolve through host bindings");

    assert_eq!(host.document_origin_calls, 2);
    assert_eq!(host.element_origin_calls, vec![ElementHandle::new(1), ElementHandle::new(2)]);
    assert_eq!(
        host.text_content.get(&ElementHandle::new(1)).map(String::as_str),
        Some(
            "https://example.test:8443:https://example.test:8443:https://example.test:8443:https://example.test:8443"
        )
    );
}

#[test]
fn runtime_resolves_query_selector_and_null_miss() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("root", ElementHandle::new(1), "scopeinside");
    host.seed_element("inside", ElementHandle::new(2), "inside");
    host.seed_element("out", ElementHandle::new(3), "");
    host.seed_document_query_selector(".primary", Some(ElementHandle::new(1)));
    host.seed_element_query_selector(
        ElementHandle::new(1),
        ".primary",
        Some(ElementHandle::new(2)),
    );
    host.seed_element_query_selector(ElementHandle::new(1), ".missing", None);

    runtime
        .eval_program(
            "const first = document.querySelector('.primary'); const scoped = document.getElementById('root').querySelector('.primary'); const missing = document.getElementById('root').querySelector('.missing'); document.getElementById('out').textContent = first.textContent + ':' + scoped.textContent + ':' + String(missing);",
            "inline-script",
            &mut host,
        )
        .expect("query selectors should resolve through host bindings");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(3))
            .map(String::as_str),
        Some("scopeinside:inside:null")
    );
    assert_eq!(
        host.document_query_selector_calls,
        vec![".primary".to_string()]
    );
    assert_eq!(
        host.element_query_selector_calls,
        vec![
            (ElementHandle::new(1), ".primary".to_string()),
            (ElementHandle::new(1), ".missing".to_string()),
        ]
    );
}

#[test]
fn runtime_resolves_query_selector_all_and_collection_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("root", ElementHandle::new(1), "root");
    host.seed_element("first", ElementHandle::new(2), "first");
    host.seed_element("out", ElementHandle::new(4), "");

    host.seed_document_query_selector_all(
        ".primary",
        vec![ElementHandle::new(1), ElementHandle::new(2)],
    );
    host.seed_element_query_selector_all(
        ElementHandle::new(1),
        ".primary",
        vec![ElementHandle::new(2)],
    );

    runtime
        .eval_program(
            "const all = document.querySelectorAll('.primary'); const scoped = document.getElementById('root').querySelectorAll('.primary'); document.getElementById('out').textContent = String(all.length) + ':' + all.item(0).textContent + ':' + all.item(1).textContent + ':' + String(all.item(2)) + ':' + String(scoped.length);",
            "inline-script",
            &mut host,
        )
        .expect("querySelectorAll should resolve");

    assert_eq!(
        host.document_query_selector_all_calls,
        vec![".primary".to_string()]
    );
    assert_eq!(
        host.element_query_selector_all_calls,
        vec![(ElementHandle::new(1), ".primary".to_string())]
    );
    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(4))
            .map(String::as_str),
        Some("2:root:first:null:1")
    );
}

#[test]
fn runtime_resolves_html_collection_children_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("root", ElementHandle::new(1), "root");
    host.seed_element("first", ElementHandle::new(2), "first");
    host.seed_element("second", ElementHandle::new(3), "second");
    host.seed_element("out", ElementHandle::new(4), "");
    host.seed_element_children(
        ElementHandle::new(1),
        vec![ElementHandle::new(2), ElementHandle::new(3)],
    );

    runtime
        .eval_program(
            "const children = document.getElementById('root').children; document.getElementById('out').textContent = String(children.length) + ':' + children.item(0).textContent + ':' + children.item(1).textContent + ':' + String(children.item(2));",
            "inline-script",
            &mut host,
        )
        .expect("HTMLCollection children should resolve");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(4))
            .map(String::as_str),
        Some("2:first:second:null")
    );
    assert_eq!(
        host.element_children_calls,
        vec![
            ElementHandle::new(1),
            ElementHandle::new(1),
            ElementHandle::new(1),
            ElementHandle::new(1),
        ]
    );
}

#[test]
fn runtime_dispatches_insert_adjacent_html() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("root", ElementHandle::new(1), "");

    runtime
        .eval_program(
            "document.getElementById('root').insertAdjacentHTML('beforeend', '<span id=\"child\">Child</span>');",
            "inline-script",
            &mut host,
        )
        .expect("insertAdjacentHTML should dispatch through host bindings");

    assert_eq!(
        host.element_insert_adjacent_html_calls,
        vec![(
            ElementHandle::new(1),
            "beforeend".to_string(),
            "<span id=\"child\">Child</span>".to_string(),
        )]
    );
}

#[test]
fn runtime_resolves_html_collection_tag_name_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("root", ElementHandle::new(1), "root");
    host.seed_element("first", ElementHandle::new(2), "first");
    host.seed_element("second", ElementHandle::new(3), "second");
    host.seed_element("out", ElementHandle::new(4), "");

    let document_collection = HtmlCollectionTarget::ByTagName {
        scope: HtmlCollectionScope::Document,
        tag_name: "span".to_string(),
    };
    let scoped_collection = HtmlCollectionTarget::ByTagName {
        scope: HtmlCollectionScope::Element(ElementHandle::new(1)),
        tag_name: "span".to_string(),
    };
    host.seed_html_collection_tag_name_items(
        document_collection.clone(),
        vec![ElementHandle::new(2), ElementHandle::new(3)],
    );
    host.seed_html_collection_tag_name_items(
        scoped_collection.clone(),
        vec![ElementHandle::new(2), ElementHandle::new(3)],
    );
    host.seed_html_collection_tag_name_named_item(
        document_collection.clone(),
        "alpha",
        Some(ElementHandle::new(2)),
    );
    host.seed_html_collection_tag_name_named_item(scoped_collection.clone(), "missing", None);

    runtime
        .eval_program(
            "const all = document.getElementsByTagName('span'); const scoped = document.getElementById('root').getElementsByTagName('span'); document.getElementById('out').textContent = String(all.length) + ':' + all.item(0).textContent + ':' + all.namedItem('alpha').textContent + ':' + String(scoped.length) + ':' + scoped.item(1).textContent + ':' + String(scoped.namedItem('missing'));",
            "inline-script",
            &mut host,
        )
        .expect("HTMLCollection getElementsByTagName should resolve");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(4))
            .map(String::as_str),
        Some("2:first:first:2:second:null")
    );
    assert_eq!(
        host.html_collection_tag_name_items_calls,
        vec![
            document_collection.clone(),
            document_collection,
            scoped_collection.clone(),
            scoped_collection
        ]
    );
    assert_eq!(
        host.html_collection_tag_name_named_item_calls,
        vec![
            (
                HtmlCollectionTarget::ByTagName {
                    scope: HtmlCollectionScope::Document,
                    tag_name: "span".to_string(),
                },
                "alpha".to_string()
            ),
            (
                HtmlCollectionTarget::ByTagName {
                    scope: HtmlCollectionScope::Element(ElementHandle::new(1)),
                    tag_name: "span".to_string(),
                },
                "missing".to_string()
            ),
        ]
    );
}

#[test]
fn runtime_resolves_html_collection_class_name_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("root", ElementHandle::new(1), "root");
    host.seed_element("first", ElementHandle::new(2), "First");
    host.seed_element("second", ElementHandle::new(3), "Second");
    host.seed_element("out", ElementHandle::new(4), "");

    let document_collection = HtmlCollectionTarget::ByClassName {
        scope: HtmlCollectionScope::Document,
        class_names: "alpha".to_string(),
    };
    let scoped_collection = HtmlCollectionTarget::ByClassName {
        scope: HtmlCollectionScope::Element(ElementHandle::new(1)),
        class_names: "alpha".to_string(),
    };
    host.seed_html_collection_class_name_items(
        document_collection.clone(),
        vec![
            ElementHandle::new(1),
            ElementHandle::new(2),
            ElementHandle::new(3),
        ],
    );
    host.seed_html_collection_class_name_items(
        scoped_collection.clone(),
        vec![ElementHandle::new(2), ElementHandle::new(3)],
    );
    host.seed_html_collection_class_name_named_item(
        document_collection.clone(),
        "alpha",
        Some(ElementHandle::new(2)),
    );
    host.seed_html_collection_class_name_named_item(scoped_collection.clone(), "alpha", None);

    runtime
        .eval_program(
            "const all = document.getElementsByClassName('alpha'); const scoped = document.getElementById('root').getElementsByClassName('alpha'); const named = all.namedItem('alpha'); const root = all.item(0); const before = all.length; const beforeScoped = scoped.length; document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(before) + ':' + String(all.length) + ':' + String(beforeScoped) + ':' + String(scoped.length) + ':' + named.textContent + ':' + String(scoped.namedItem('alpha')) + ':' + root.textContent;",
            "inline-script",
            &mut host,
        )
        .expect("HTMLCollection getElementsByClassName should resolve");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(4))
            .map(String::as_str),
        Some("3:3:2:2:First:null:gone")
    );
    assert_eq!(
        host.html_collection_class_name_items_calls,
        vec![
            document_collection.clone(),
            document_collection.clone(),
            scoped_collection.clone(),
            document_collection,
            scoped_collection,
        ]
    );
    assert_eq!(
        host.html_collection_class_name_named_item_calls,
        vec![
            (
                HtmlCollectionTarget::ByClassName {
                    scope: HtmlCollectionScope::Document,
                    class_names: "alpha".to_string(),
                },
                "alpha".to_string()
            ),
            (
                HtmlCollectionTarget::ByClassName {
                    scope: HtmlCollectionScope::Element(ElementHandle::new(1)),
                    class_names: "alpha".to_string(),
                },
                "alpha".to_string()
            ),
        ]
    );
}

#[test]
fn runtime_resolves_document_forms_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("signup", ElementHandle::new(1), "Signup");
    host.seed_element("login", ElementHandle::new(2), "Login");
    host.seed_element("out", ElementHandle::new(3), "");

    let forms_collection = HtmlCollectionTarget::ByTagName {
        scope: HtmlCollectionScope::Document,
        tag_name: "form".to_string(),
    };
    host.seed_html_collection_tag_name_items(
        forms_collection.clone(),
        vec![ElementHandle::new(1), ElementHandle::new(2)],
    );
    host.seed_html_collection_tag_name_named_item(
        forms_collection.clone(),
        "signup",
        Some(ElementHandle::new(1)),
    );
    host.seed_html_collection_tag_name_named_item(forms_collection.clone(), "missing", None);

    runtime
        .eval_program(
            "const forms = document.forms; const named = forms.namedItem('signup'); document.getElementById('out').textContent = String(forms.length) + ':' + forms.item(0).textContent + ':' + named.textContent + ':' + String(forms.namedItem('missing'));",
            "inline-script",
            &mut host,
        )
        .expect("document.forms should resolve");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(3))
            .map(String::as_str),
        Some("2:Signup:Signup:null")
    );
    assert_eq!(
        host.html_collection_tag_name_items_calls,
        vec![forms_collection.clone(), forms_collection.clone()]
    );
    assert_eq!(
        host.html_collection_tag_name_named_item_calls,
        vec![
            (
                HtmlCollectionTarget::ByTagName {
                    scope: HtmlCollectionScope::Document,
                    tag_name: "form".to_string(),
                },
                "signup".to_string()
            ),
            (
                HtmlCollectionTarget::ByTagName {
                    scope: HtmlCollectionScope::Document,
                    tag_name: "form".to_string(),
                },
                "missing".to_string()
            ),
        ]
    );
}

#[test]
fn runtime_resolves_form_elements_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("signup", ElementHandle::new(1), "Signup");
    host.seed_element("first", ElementHandle::new(2), "Ada");
    host.seed_element("second", ElementHandle::new(3), "Bio");
    host.seed_element("out", ElementHandle::new(4), "");
    host.seed_value(ElementHandle::new(2), "Ada");
    host.seed_value(ElementHandle::new(3), "Bio");

    host.seed_html_collection_form_elements_items(
        ElementHandle::new(1),
        vec![ElementHandle::new(2), ElementHandle::new(3)],
    );
    host.seed_html_collection_form_elements_named_item(
        ElementHandle::new(1),
        "first",
        Some(ElementHandle::new(2)),
    );
    host.seed_html_collection_form_elements_named_item(ElementHandle::new(1), "missing", None);

    runtime
        .eval_program(
            "const elements = document.getElementById('signup').elements; const named = elements.namedItem('first'); document.getElementById('out').textContent = String(elements.length) + ':' + elements.item(0).value + ':' + elements.item(1).value + ':' + named.value + ':' + String(elements.namedItem('missing'));",
            "inline-script",
            &mut host,
        )
        .expect("form elements should resolve");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(4))
            .map(String::as_str),
        Some("2:Ada:Bio:Ada:null")
    );
    assert_eq!(
        host.html_collection_form_elements_items_calls,
        vec![
            ElementHandle::new(1),
            ElementHandle::new(1),
            ElementHandle::new(1)
        ]
    );
    assert_eq!(
        host.html_collection_form_elements_named_item_calls,
        vec![
            (ElementHandle::new(1), "first".to_string()),
            (ElementHandle::new(1), "missing".to_string()),
        ]
    );
}

#[test]
fn runtime_resolves_form_elements_radio_node_list_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("signup", ElementHandle::new(1), "Signup");
    host.seed_element("radio-a", ElementHandle::new(2), "");
    host.seed_element("radio-b", ElementHandle::new(3), "");
    host.seed_element("out", ElementHandle::new(4), "");
    host.seed_value(ElementHandle::new(2), "a");
    host.seed_value(ElementHandle::new(3), "b");
    host.seed_checked(ElementHandle::new(2), true);
    host.seed_checked(ElementHandle::new(3), false);

    host.seed_html_collection_form_elements_items(
        ElementHandle::new(1),
        vec![ElementHandle::new(2), ElementHandle::new(3)],
    );
    host.seed_html_collection_form_elements_named_items(
        ElementHandle::new(1),
        "mode",
        vec![ElementHandle::new(2), ElementHandle::new(3)],
    );

    runtime
        .eval_program(
            "const elements = document.getElementById('signup').elements; const named = elements.namedItem('mode'); document.getElementById('out').textContent = String(elements.length) + ':' + String(named.length) + ':' + named.item(0).value + ':' + named.item(1).value + ':' + named.value + ':' + String(named);",
            "inline-script",
            &mut host,
        )
        .expect("radio node list should resolve");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(4))
            .map(String::as_str),
        Some("2:2:a:b:a:[object RadioNodeList]")
    );
    assert_eq!(
        host.html_collection_form_elements_named_items_calls,
        vec![
            (ElementHandle::new(1), "mode".to_string()),
            (ElementHandle::new(1), "mode".to_string()),
            (ElementHandle::new(1), "mode".to_string()),
            (ElementHandle::new(1), "mode".to_string()),
            (ElementHandle::new(1), "mode".to_string()),
        ]
    );
}

#[test]
fn runtime_resolves_select_options_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("mode", ElementHandle::new(1), "mode");
    host.seed_element("first", ElementHandle::new(2), "A");
    host.seed_element("second", ElementHandle::new(3), "B");
    host.seed_element("out", ElementHandle::new(4), "");
    host.seed_html_collection_select_options_items(
        ElementHandle::new(1),
        vec![ElementHandle::new(2), ElementHandle::new(3)],
    );
    host.seed_html_collection_select_options_named_item(
        ElementHandle::new(1),
        "second",
        Some(ElementHandle::new(3)),
    );
    host.seed_html_collection_select_options_named_item(ElementHandle::new(1), "missing", None);

    runtime
        .eval_program(
            "const options = document.getElementById('mode').options; const named = options.namedItem('second'); const before = options.length; document.getElementById('out').textContent = String(before) + ':' + options.item(0).textContent + ':' + options.item(1).textContent + ':' + named.textContent + ':' + String(options.namedItem('missing'));",
            "inline-script",
            &mut host,
        )
        .expect("select.options should resolve");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(4))
            .map(String::as_str),
        Some("2:A:B:B:null")
    );
    assert_eq!(
        host.html_collection_select_options_items_calls,
        vec![
            ElementHandle::new(1),
            ElementHandle::new(1),
            ElementHandle::new(1),
        ]
    );
    assert_eq!(
        host.html_collection_select_options_named_item_calls,
        vec![
            (ElementHandle::new(1), "second".to_string()),
            (ElementHandle::new(1), "missing".to_string()),
        ]
    );
}

#[test]
fn runtime_resolves_fieldset_elements_and_datalist_options_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("fieldset", ElementHandle::new(1), "fieldset");
    host.seed_element("first-control", ElementHandle::new(2), "A");
    host.seed_element("second-control", ElementHandle::new(3), "B");
    host.seed_element("list", ElementHandle::new(4), "list");
    host.seed_element("first-option", ElementHandle::new(5), "One");
    host.seed_element("second-option", ElementHandle::new(6), "Two");
    host.seed_element("out", ElementHandle::new(7), "");
    host.seed_html_collection_form_elements_items(
        ElementHandle::new(1),
        vec![ElementHandle::new(2), ElementHandle::new(3)],
    );
    host.seed_html_collection_form_elements_named_item(
        ElementHandle::new(1),
        "second-control",
        Some(ElementHandle::new(3)),
    );
    host.seed_html_collection_form_elements_named_item(ElementHandle::new(1), "missing", None);
    host.seed_html_collection_select_options_items(
        ElementHandle::new(4),
        vec![ElementHandle::new(5), ElementHandle::new(6)],
    );
    host.seed_html_collection_select_options_named_item(
        ElementHandle::new(4),
        "second-option",
        Some(ElementHandle::new(6)),
    );
    host.seed_html_collection_select_options_named_item(ElementHandle::new(4), "missing", None);

    runtime
        .eval_program(
            "const controls = document.getElementById('fieldset').elements; const options = document.getElementById('list').options; document.getElementById('out').textContent = String(controls.length) + ':' + String(options.length) + ':' + controls.item(0).textContent + ':' + controls.item(1).textContent + ':' + String(controls.namedItem('second-control')) + ':' + options.item(0).textContent + ':' + options.item(1).textContent + ':' + options.namedItem('second-option').textContent + ':' + String(options.namedItem('missing'));",
            "inline-script",
            &mut host,
        )
        .expect("fieldset.elements and datalist.options should resolve");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(7))
            .map(String::as_str),
        Some("2:2:A:B:[object Element]:One:Two:Two:null")
    );
    assert_eq!(
        host.html_collection_form_elements_items_calls,
        vec![
            ElementHandle::new(1),
            ElementHandle::new(1),
            ElementHandle::new(1),
        ]
    );
    assert_eq!(
        host.html_collection_form_elements_named_item_calls,
        vec![(ElementHandle::new(1), "second-control".to_string())]
    );
    assert_eq!(
        host.html_collection_select_options_items_calls,
        vec![
            ElementHandle::new(4),
            ElementHandle::new(4),
            ElementHandle::new(4),
        ]
    );
    assert_eq!(
        host.html_collection_select_options_named_item_calls,
        vec![
            (ElementHandle::new(4), "second-option".to_string()),
            (ElementHandle::new(4), "missing".to_string()),
        ]
    );
}

#[test]
fn runtime_resolves_select_selected_options_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("mode", ElementHandle::new(1), "mode");
    host.seed_element("first", ElementHandle::new(2), "A");
    host.seed_element("second", ElementHandle::new(3), "B");
    host.seed_element("out", ElementHandle::new(4), "");
    host.seed_html_collection_select_selected_options_items(
        ElementHandle::new(1),
        vec![ElementHandle::new(2), ElementHandle::new(3)],
    );
    host.seed_html_collection_select_selected_options_named_item(
        ElementHandle::new(1),
        "second",
        Some(ElementHandle::new(3)),
    );
    host.seed_html_collection_select_selected_options_named_item(
        ElementHandle::new(1),
        "missing",
        None,
    );

    runtime
        .eval_program(
            "const selected = document.getElementById('mode').selectedOptions; const named = selected.namedItem('second'); const before = selected.length; document.getElementById('out').textContent = String(before) + ':' + selected.item(0).textContent + ':' + selected.item(1).textContent + ':' + named.textContent + ':' + String(selected.namedItem('missing'));",
            "inline-script",
            &mut host,
        )
        .expect("select.selectedOptions should resolve");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(4))
            .map(String::as_str),
        Some("2:A:B:B:null")
    );
    assert_eq!(
        host.html_collection_select_selected_options_items_calls,
        vec![
            ElementHandle::new(1),
            ElementHandle::new(1),
            ElementHandle::new(1),
        ]
    );
    assert_eq!(
        host.html_collection_select_selected_options_named_item_calls,
        vec![
            (ElementHandle::new(1), "second".to_string()),
            (ElementHandle::new(1), "missing".to_string()),
        ]
    );
}

#[test]
fn runtime_resolves_element_labels_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("control", ElementHandle::new(1), "");
    host.seed_element("explicit-label", ElementHandle::new(2), "Explicit");
    host.seed_element("implicit-label", ElementHandle::new(3), "Implicit");
    host.seed_element("inner-control", ElementHandle::new(4), "");
    host.seed_element("wrapper", ElementHandle::new(5), "");
    host.seed_element("out", ElementHandle::new(6), "");
    host.seed_element("second-label", ElementHandle::new(7), "Second");
    host.seed_attribute(ElementHandle::new(2), "id", "explicit-label");
    host.seed_attribute(ElementHandle::new(3), "id", "implicit-label");
    host.seed_attribute(ElementHandle::new(7), "id", "second-label");
    host.seed_element_labels(
        ElementHandle::new(1),
        vec![ElementHandle::new(2), ElementHandle::new(7)],
    );
    host.seed_element_labels(ElementHandle::new(4), vec![ElementHandle::new(3)]);

    runtime
        .eval_program(
            "const control = document.getElementById('control'); const labels = control.labels; const inner = document.getElementById('inner-control').labels; document.getElementById('wrapper').textContent = 'updated'; document.getElementById('out').textContent = String(labels.length) + ':' + String(labels.length) + ':' + labels.item(0).getAttribute('id') + ':' + labels.item(1).textContent + ':' + String(inner.length) + ':' + inner.item(0).getAttribute('id');",
            "inline-script",
            &mut host,
        )
        .expect("label collections should resolve");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(6))
            .map(String::as_str),
        Some("2:2:explicit-label:Second:1:implicit-label")
    );
    assert_eq!(
        host.element_labels_calls,
        vec![
            ElementHandle::new(1),
            ElementHandle::new(1),
            ElementHandle::new(1),
            ElementHandle::new(1),
            ElementHandle::new(4),
            ElementHandle::new(4),
        ]
    );
}

#[test]
fn runtime_resolves_map_areas_and_table_bodies_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("map", ElementHandle::new(1), "map");
    host.seed_element("first-area", ElementHandle::new(2), "First area");
    host.seed_element("second-area", ElementHandle::new(3), "Second area");
    host.seed_element("table", ElementHandle::new(4), "table");
    host.seed_element("first-body", ElementHandle::new(5), "Body 1");
    host.seed_element("second-body", ElementHandle::new(6), "Body 2");
    host.seed_element("out", ElementHandle::new(7), "");
    host.seed_html_collection_map_areas_items(
        ElementHandle::new(1),
        vec![ElementHandle::new(2), ElementHandle::new(3)],
    );
    host.seed_html_collection_map_areas_named_item(
        ElementHandle::new(1),
        "second-area",
        Some(ElementHandle::new(3)),
    );
    host.seed_html_collection_map_areas_named_item(ElementHandle::new(1), "missing", None);
    host.seed_html_collection_table_bodies_items(
        ElementHandle::new(4),
        vec![ElementHandle::new(5), ElementHandle::new(6)],
    );
    host.seed_html_collection_table_bodies_named_item(
        ElementHandle::new(4),
        "second-body",
        Some(ElementHandle::new(6)),
    );
    host.seed_html_collection_table_bodies_named_item(ElementHandle::new(4), "missing", None);

    runtime
        .eval_program(
            "const areas = document.getElementById('map').areas; const bodies = document.getElementById('table').tBodies; document.getElementById('out').textContent = String(areas.length) + ':' + String(bodies.length) + ':' + areas.item(0).textContent + ':' + bodies.item(0).textContent + ':' + String(areas.namedItem('second-area')) + ':' + String(areas.namedItem('missing')) + ':' + String(bodies.namedItem('second-body')) + ':' + String(bodies.namedItem('missing'));",
            "inline-script",
            &mut host,
        )
        .expect("map.areas and table.tBodies should resolve");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(7))
            .map(String::as_str),
        Some("2:2:First area:Body 1:[object Element]:null:[object Element]:null")
    );
    assert_eq!(
        host.html_collection_map_areas_items_calls,
        vec![ElementHandle::new(1), ElementHandle::new(1)]
    );
    assert_eq!(
        host.html_collection_map_areas_named_item_calls,
        vec![
            (ElementHandle::new(1), "second-area".to_string()),
            (ElementHandle::new(1), "missing".to_string()),
        ]
    );
    assert_eq!(
        host.html_collection_table_bodies_items_calls,
        vec![ElementHandle::new(4), ElementHandle::new(4)]
    );
    assert_eq!(
        host.html_collection_table_bodies_named_item_calls,
        vec![
            (ElementHandle::new(4), "second-body".to_string()),
            (ElementHandle::new(4), "missing".to_string()),
        ]
    );
}

#[test]
fn runtime_resolves_document_images_and_links_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("img-hero", ElementHandle::new(1), "");
    host.seed_element("img-thumb", ElementHandle::new(2), "");
    host.seed_element("docs", ElementHandle::new(3), "Docs");
    host.seed_element("map", ElementHandle::new(4), "");
    host.seed_element("out", ElementHandle::new(5), "");
    host.seed_html_collection_tag_name_items(
        HtmlCollectionTarget::ByTagName {
            scope: HtmlCollectionScope::Document,
            tag_name: "img".to_string(),
        },
        vec![ElementHandle::new(1), ElementHandle::new(2)],
    );
    host.seed_html_collection_tag_name_named_item(
        HtmlCollectionTarget::ByTagName {
            scope: HtmlCollectionScope::Document,
            tag_name: "img".to_string(),
        },
        "img-hero",
        Some(ElementHandle::new(1)),
    );
    host.seed_html_collection_tag_name_named_item(
        HtmlCollectionTarget::ByTagName {
            scope: HtmlCollectionScope::Document,
            tag_name: "img".to_string(),
        },
        "img-thumb",
        Some(ElementHandle::new(2)),
    );
    host.seed_document_links_items(vec![ElementHandle::new(3), ElementHandle::new(4)]);
    host.seed_document_links_named_item("docs", Some(ElementHandle::new(3)));
    host.seed_document_links_named_item("map", Some(ElementHandle::new(4)));
    host.seed_document_links_named_item("plain", None);

    runtime
        .eval_program(
            "const images = document.images; const links = document.links; const beforeImages = images.length; const beforeLinks = links.length; const hero = images.namedItem('img-hero'); const thumb = images.namedItem('img-thumb'); const docs = links.namedItem('docs'); const map = links.namedItem('map'); document.getElementById('out').textContent = String(beforeImages) + ':' + String(images.length) + ':' + String(beforeLinks) + ':' + String(links.length) + ':' + String(hero) + ':' + String(thumb) + ':' + String(docs) + ':' + String(map) + ':' + String(links.namedItem('plain'));",
            "inline-script",
            &mut host,
        )
        .expect("document.images and document.links should resolve");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(5))
            .map(String::as_str),
        Some("2:2:2:2:[object Element]:[object Element]:[object Element]:[object Element]:null")
    );
    assert_eq!(
        host.html_collection_tag_name_items_calls,
        vec![
            HtmlCollectionTarget::ByTagName {
                scope: HtmlCollectionScope::Document,
                tag_name: "img".to_string(),
            },
            HtmlCollectionTarget::ByTagName {
                scope: HtmlCollectionScope::Document,
                tag_name: "img".to_string(),
            },
        ]
    );
    assert_eq!(
        host.html_collection_tag_name_named_item_calls,
        vec![
            (
                HtmlCollectionTarget::ByTagName {
                    scope: HtmlCollectionScope::Document,
                    tag_name: "img".to_string(),
                },
                "img-hero".to_string()
            ),
            (
                HtmlCollectionTarget::ByTagName {
                    scope: HtmlCollectionScope::Document,
                    tag_name: "img".to_string(),
                },
                "img-thumb".to_string()
            ),
        ]
    );
    assert_eq!(host.document_links_items_calls, 2);
    assert_eq!(
        host.document_links_named_item_calls,
        vec!["docs".to_string(), "map".to_string(), "plain".to_string()]
    );
}

#[test]
fn runtime_resolves_document_anchors_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("anchor-one", ElementHandle::new(1), "First");
    host.seed_element("anchor-two", ElementHandle::new(2), "Second");
    host.seed_element("out", ElementHandle::new(3), "");
    host.seed_document_anchors_items(vec![ElementHandle::new(1), ElementHandle::new(2)]);
    host.seed_document_anchors_named_item("first", Some(ElementHandle::new(1)));
    host.seed_document_anchors_named_item("second", Some(ElementHandle::new(2)));
    host.seed_document_anchors_named_item("missing", None);

    runtime
        .eval_program(
            "const anchors = document.anchors; document.getElementById('out').textContent = String(anchors.length) + ':' + String(anchors.length) + ':' + anchors.item(0).textContent + ':' + anchors.namedItem('first').textContent + ':' + anchors.namedItem('second').textContent + ':' + String(anchors.namedItem('missing'));",
            "inline-script",
            &mut host,
        )
        .expect("document.anchors should resolve");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(3))
            .map(String::as_str),
        Some("2:2:First:First:Second:null")
    );
    assert_eq!(host.document_anchors_items_calls, 3);
    assert_eq!(
        host.document_anchors_named_item_calls,
        vec![
            "first".to_string(),
            "second".to_string(),
            "missing".to_string()
        ]
    );
}

#[test]
fn runtime_resolves_document_children_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("root", ElementHandle::new(1), "First");
    host.seed_element("out", ElementHandle::new(2), "Second");
    host.seed_document_children_items(vec![ElementHandle::new(1), ElementHandle::new(2)]);
    host.seed_document_children_named_item("root", Some(ElementHandle::new(1)));
    host.seed_document_children_named_item("missing", None);

    runtime
        .eval_program(
            "const children = document.children; document.getElementById('out').textContent = String(children.length) + ':' + children.item(0).textContent + ':' + children.item(1).textContent + ':' + String(children.namedItem('root')) + ':' + String(children.namedItem('missing'));",
            "inline-script",
            &mut host,
        )
        .expect("document.children should resolve");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(2))
            .map(String::as_str),
        Some("2:First:Second:[object Element]:null")
    );
    assert_eq!(host.document_children_items_calls, 3);
    assert_eq!(
        host.document_children_named_item_calls,
        vec!["root".to_string(), "missing".to_string()]
    );
}

#[test]
fn runtime_resolves_child_nodes_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("root", ElementHandle::new(1), "");
    host.seed_element("out", ElementHandle::new(2), "");
    host.seed_node_child_nodes_items(
        HtmlCollectionScope::Document,
        vec![NodeHandle::new(10), NodeHandle::new(11)],
    );
    host.seed_node_child_nodes_items(
        HtmlCollectionScope::Element(ElementHandle::new(1)),
        vec![
            NodeHandle::new(20),
            NodeHandle::new(21),
            NodeHandle::new(22),
        ],
    );
    host.seed_node_name(NodeHandle::new(10), "#comment");
    host.seed_node_type(NodeHandle::new(10), 8);
    host.seed_node_text_content(NodeHandle::new(10), "");
    host.seed_node_name(NodeHandle::new(11), "main");
    host.seed_node_type(NodeHandle::new(11), 1);
    host.seed_node_text_content(NodeHandle::new(11), "Root");
    host.seed_node_name(NodeHandle::new(20), "#text");
    host.seed_node_type(NodeHandle::new(20), 3);
    host.seed_node_text_content(NodeHandle::new(20), "Hello");
    host.seed_node_name(NodeHandle::new(21), "span");
    host.seed_node_type(NodeHandle::new(21), 1);
    host.seed_node_text_content(NodeHandle::new(21), "Inner");
    host.seed_node_name(NodeHandle::new(22), "#comment");
    host.seed_node_type(NodeHandle::new(22), 8);
    host.seed_node_text_content(NodeHandle::new(22), "");

    runtime
        .eval_program(
            "const docNodes = document.childNodes; const rootNodes = document.getElementById('root').childNodes; const docFirst = docNodes.item(0); const docSecond = docNodes.item(1); const rootValues = rootNodes.values(); const firstRoot = rootValues.next(); const secondRoot = rootValues.next(); const thirdRoot = rootValues.next(); document.getElementById('out').textContent = String(docNodes.length) + ':' + docFirst.nodeName + ':' + String(docFirst.nodeType) + ':' + String(docFirst) + ':' + docSecond.nodeName + ':' + String(docSecond.nodeType) + ':' + firstRoot.value.nodeName + ':' + String(firstRoot.value.nodeType) + ':' + firstRoot.value.textContent + ':' + secondRoot.value.nodeName + ':' + String(secondRoot.value.nodeType) + ':' + secondRoot.value.textContent + ':' + thirdRoot.value.nodeName + ':' + String(thirdRoot.value.nodeType) + ':' + thirdRoot.value.textContent;",
            "inline-script",
            &mut host,
        )
        .expect("childNodes should resolve");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(2))
            .map(String::as_str),
        Some("2:#comment:8:[object Node]:main:1:#text:3:Hello:span:1:Inner:#comment:8:")
    );
    assert_eq!(
        host.node_child_nodes_items_calls,
        vec![
            HtmlCollectionScope::Document,
            HtmlCollectionScope::Document,
            HtmlCollectionScope::Element(ElementHandle::new(1)),
            HtmlCollectionScope::Document
        ]
    );
    assert_eq!(
        host.node_name_calls,
        vec![
            NodeHandle::new(10),
            NodeHandle::new(11),
            NodeHandle::new(20),
            NodeHandle::new(21),
            NodeHandle::new(22)
        ]
    );
    assert_eq!(
        host.node_type_calls,
        vec![
            NodeHandle::new(10),
            NodeHandle::new(11),
            NodeHandle::new(20),
            NodeHandle::new(21),
            NodeHandle::new(22)
        ]
    );
    assert_eq!(
        host.node_text_content_calls,
        vec![
            NodeHandle::new(20),
            NodeHandle::new(21),
            NodeHandle::new(22)
        ]
    );
}

#[test]
fn runtime_resolves_table_rows_and_row_cells_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("table", ElementHandle::new(1), "Table");
    host.seed_element("body", ElementHandle::new(2), "Body");
    host.seed_element("row", ElementHandle::new(3), "Row");
    host.seed_element("cell", ElementHandle::new(4), "Cell");
    host.seed_table_rows_items(
        ElementHandle::new(1),
        vec![ElementHandle::new(3), ElementHandle::new(5)],
    );
    host.seed_table_rows_items(ElementHandle::new(2), vec![ElementHandle::new(3)]);
    host.seed_row_cells_items(ElementHandle::new(3), vec![ElementHandle::new(4)]);
    host.seed_table_rows_named_item(ElementHandle::new(1), "missing", None);
    host.seed_table_rows_named_item(ElementHandle::new(2), "row", Some(ElementHandle::new(3)));
    host.seed_row_cells_named_item(ElementHandle::new(3), "cell", Some(ElementHandle::new(4)));

    runtime
        .eval_program(
            "const table = document.getElementById('table'); const body = document.getElementById('body'); const row = document.getElementById('row'); document.getElementById('cell').textContent = String(table.rows.length) + ':' + String(body.rows.length) + ':' + String(row.cells.length) + ':' + String(table.rows.item(0)) + ':' + String(body.rows.namedItem('row')) + ':' + String(row.cells.namedItem('cell')) + ':' + String(table.rows.namedItem('missing'));",
            "inline-script",
            &mut host,
        )
        .expect("table rows and row cells should resolve");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(4))
            .map(String::as_str),
        Some("2:1:1:[object Element]:[object Element]:[object Element]:null")
    );
    assert_eq!(
        host.table_rows_items_calls,
        vec![
            ElementHandle::new(1),
            ElementHandle::new(2),
            ElementHandle::new(1)
        ]
    );
    assert_eq!(host.row_cells_items_calls, vec![ElementHandle::new(3)]);
    assert_eq!(
        host.table_rows_named_item_calls,
        vec![
            (ElementHandle::new(2), "row".to_string()),
            (ElementHandle::new(1), "missing".to_string())
        ]
    );
    assert_eq!(
        host.row_cells_named_item_calls,
        vec![(ElementHandle::new(3), "cell".to_string())]
    );
}

#[test]
fn runtime_resolves_document_scripts_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("first-script", ElementHandle::new(1), "First");
    host.seed_element("second-script", ElementHandle::new(2), "Second");
    host.seed_element("out", ElementHandle::new(3), "");
    let scripts_collection = HtmlCollectionTarget::ByTagName {
        scope: HtmlCollectionScope::Document,
        tag_name: "script".to_string(),
    };
    host.seed_html_collection_tag_name_items(
        scripts_collection.clone(),
        vec![ElementHandle::new(1), ElementHandle::new(2)],
    );
    host.seed_html_collection_tag_name_named_item(
        scripts_collection.clone(),
        "first-script",
        Some(ElementHandle::new(1)),
    );
    host.seed_html_collection_tag_name_named_item(scripts_collection.clone(), "missing", None);

    runtime
        .eval_program(
            "const scripts = document.scripts; document.getElementById('out').textContent = String(scripts.length) + ':' + scripts.item(0).textContent + ':' + scripts.namedItem('first-script').textContent + ':' + String(scripts.namedItem('missing'));",
            "inline-script",
            &mut host,
        )
        .expect("document.scripts should resolve");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(3))
            .map(String::as_str),
        Some("2:First:First:null")
    );
    assert_eq!(
        host.html_collection_tag_name_items_calls,
        vec![scripts_collection.clone(), scripts_collection.clone()]
    );
    assert_eq!(
        host.html_collection_tag_name_named_item_calls,
        vec![
            (scripts_collection.clone(), "first-script".to_string()),
            (scripts_collection, "missing".to_string()),
        ]
    );
}

#[test]
fn runtime_resolves_template_content_child_nodes_and_children_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("tpl", ElementHandle::new(3), "");
    host.seed_element("out", ElementHandle::new(2), "");
    host.seed_element_tag_name(ElementHandle::new(3), "template");
    host.seed_element("inner", ElementHandle::new(40), "Inner");
    host.seed_node_child_nodes_items(
        HtmlCollectionScope::Element(ElementHandle::new(3)),
        vec![
            NodeHandle::new(30),
            NodeHandle::new(31),
            NodeHandle::new(32),
        ],
    );
    host.seed_element_children(ElementHandle::new(3), vec![ElementHandle::new(40)]);
    host.seed_html_collection_named_item(
        ElementHandle::new(3),
        "inner",
        Some(ElementHandle::new(40)),
    );
    host.seed_node_name(NodeHandle::new(30), "#text");
    host.seed_node_type(NodeHandle::new(30), 3);
    host.seed_node_text_content(NodeHandle::new(30), "Before");
    host.seed_node_name(NodeHandle::new(31), "span");
    host.seed_node_type(NodeHandle::new(31), 1);
    host.seed_node_text_content(NodeHandle::new(31), "Inner");
    host.seed_node_name(NodeHandle::new(32), "#comment");
    host.seed_node_type(NodeHandle::new(32), 8);
    host.seed_node_text_content(NodeHandle::new(32), "");

    runtime
        .eval_program(
            "const tpl = document.getElementById('tpl'); const content = tpl.content; const nodes = content.childNodes; const children = content.children; const first = nodes.item(0); const second = nodes.item(1); const third = nodes.item(2); document.getElementById('inner'); document.getElementById('out').textContent = String(content) + ':' + String(nodes.length) + ':' + first.nodeName + ':' + String(first.nodeType) + ':' + second.nodeName + ':' + String(second.nodeType) + ':' + third.nodeName + ':' + String(third.nodeType) + ':' + String(children.length) + ':' + children.item(0).textContent + ':' + String(children.namedItem('inner'));",
            "inline-script",
            &mut host,
        )
        .expect("template content collections should resolve");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(2))
            .map(String::as_str),
        Some("[object DocumentFragment]:3:#text:3:span:1:#comment:8:1:Inner:[object Element]")
    );
    assert_eq!(
        host.node_child_nodes_items_calls,
        vec![
            HtmlCollectionScope::Element(ElementHandle::new(3)),
            HtmlCollectionScope::Element(ElementHandle::new(3)),
            HtmlCollectionScope::Element(ElementHandle::new(3)),
            HtmlCollectionScope::Element(ElementHandle::new(3))
        ]
    );
    assert_eq!(host.element_tag_name_calls, vec![ElementHandle::new(3)]);
    assert_eq!(
        host.element_children_calls,
        vec![ElementHandle::new(3), ElementHandle::new(3)]
    );
}

#[test]
fn runtime_resolves_template_content_inner_html_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("tpl", ElementHandle::new(3), "");
    host.seed_element("out", ElementHandle::new(2), "");
    host.seed_element_tag_name(ElementHandle::new(3), "template");
    host.seed_element_inner_html(ElementHandle::new(3), "<span id='inner'>Inner</span>");

    runtime
        .eval_program(
            "const tpl = document.getElementById('tpl'); const content = tpl.content; const before = content.innerHTML; content.innerHTML = '<!--tail--><span id=\"second\">Second</span>'; document.getElementById('out').textContent = before + ':' + content.innerHTML;",
            "inline-script",
            &mut host,
        )
        .expect("template content innerHTML should resolve");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(2))
            .map(String::as_str),
        Some("<span id='inner'>Inner</span>:<!--tail--><span id=\"second\">Second</span>")
    );
    assert_eq!(
        host.element_inner_html_calls,
        vec![ElementHandle::new(3), ElementHandle::new(3)]
    );
    assert_eq!(
        host.element_set_inner_html_calls,
        vec![(
            ElementHandle::new(3),
            "<!--tail--><span id=\"second\">Second</span>".to_string(),
        )]
    );
    assert_eq!(host.element_tag_name_calls, vec![ElementHandle::new(3)]);
}

#[test]
fn runtime_resolves_document_style_sheets_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(3), "");
    host.seed_document_style_sheets_items(vec![ElementHandle::new(1), ElementHandle::new(2)]);

    runtime
        .eval_program(
            "const sheets = document.styleSheets; document.getElementById('out').textContent = String(sheets.length) + ':' + String(sheets.item(0)) + ':' + String(sheets.item(2));",
            "inline-script",
            &mut host,
        )
        .expect("document.styleSheets should resolve");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(3))
            .map(String::as_str),
        Some("2:[object CSSStyleSheet]:null")
    );
    assert_eq!(host.document_style_sheets_items_calls, 3);
}

#[test]
fn runtime_resolves_document_applets_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("first-applet", ElementHandle::new(1), "First");
    host.seed_element("second-applet", ElementHandle::new(2), "Second");
    host.seed_element("out", ElementHandle::new(3), "");
    let applets_collection = HtmlCollectionTarget::ByTagName {
        scope: HtmlCollectionScope::Document,
        tag_name: "applet".to_string(),
    };
    host.seed_html_collection_tag_name_items(
        applets_collection.clone(),
        vec![ElementHandle::new(1), ElementHandle::new(2)],
    );
    host.seed_html_collection_tag_name_named_item(
        applets_collection.clone(),
        "first-applet",
        Some(ElementHandle::new(1)),
    );
    host.seed_html_collection_tag_name_named_item(applets_collection.clone(), "missing", None);

    runtime
        .eval_program(
            "const applets = document.applets; document.getElementById('out').textContent = String(applets.length) + ':' + applets.item(0).textContent + ':' + String(applets.namedItem('first-applet')) + ':' + String(applets.namedItem('missing'));",
            "inline-script",
            &mut host,
        )
        .expect("document.applets should resolve");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(3))
            .map(String::as_str),
        Some("2:First:[object Element]:null")
    );
    assert_eq!(
        host.html_collection_tag_name_items_calls,
        vec![applets_collection.clone(), applets_collection.clone()]
    );
    assert_eq!(
        host.html_collection_tag_name_named_item_calls,
        vec![
            (applets_collection.clone(), "first-applet".to_string()),
            (applets_collection, "missing".to_string()),
        ]
    );
}

#[test]
fn runtime_resolves_document_embeds_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("first-embed", ElementHandle::new(1), "");
    host.seed_element("second-embed", ElementHandle::new(2), "");
    host.seed_element("out", ElementHandle::new(3), "");
    let embeds_collection = HtmlCollectionTarget::ByTagName {
        scope: HtmlCollectionScope::Document,
        tag_name: "embed".to_string(),
    };
    host.seed_html_collection_tag_name_items(
        embeds_collection.clone(),
        vec![ElementHandle::new(1), ElementHandle::new(2)],
    );
    host.seed_html_collection_tag_name_named_item(
        embeds_collection.clone(),
        "first-embed",
        Some(ElementHandle::new(1)),
    );
    host.seed_html_collection_tag_name_named_item(embeds_collection.clone(), "missing", None);

    runtime
        .eval_program(
            "const embeds = document.embeds; document.getElementById('out').textContent = String(embeds.length) + ':' + String(embeds.item(0)) + ':' + String(embeds.namedItem('first-embed')) + ':' + String(embeds.namedItem('missing'));",
            "inline-script",
            &mut host,
        )
        .expect("document.embeds should resolve");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(3))
            .map(String::as_str),
        Some("2:[object Element]:[object Element]:null")
    );
    assert_eq!(
        host.html_collection_tag_name_items_calls,
        vec![embeds_collection.clone(), embeds_collection.clone()]
    );
    assert_eq!(
        host.html_collection_tag_name_named_item_calls,
        vec![
            (embeds_collection.clone(), "first-embed".to_string()),
            (embeds_collection, "missing".to_string()),
        ]
    );
}

#[test]
fn runtime_resolves_document_all_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("root", ElementHandle::new(1), "root");
    host.seed_element("first", ElementHandle::new(2), "First");
    host.seed_element("second", ElementHandle::new(3), "Second");
    host.seed_element("out", ElementHandle::new(4), "");
    let all_collection = HtmlCollectionTarget::ByTagName {
        scope: HtmlCollectionScope::Document,
        tag_name: "*".to_string(),
    };
    host.seed_html_collection_tag_name_items(
        all_collection.clone(),
        vec![
            ElementHandle::new(1),
            ElementHandle::new(2),
            ElementHandle::new(3),
        ],
    );
    host.seed_html_collection_tag_name_named_item(
        all_collection.clone(),
        "root",
        Some(ElementHandle::new(1)),
    );
    host.seed_html_collection_tag_name_named_item(
        all_collection.clone(),
        "second",
        Some(ElementHandle::new(3)),
    );
    host.seed_html_collection_tag_name_named_item(all_collection.clone(), "missing", None);

    runtime
        .eval_program(
            "const all = document.all; const before = all.length; const named = all.namedItem('second'); document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(before) + ':' + String(all.length) + ':' + String(named) + ':' + String(all.namedItem('missing'));",
            "inline-script",
            &mut host,
        )
        .expect("document.all should resolve");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(4))
            .map(String::as_str),
        Some("3:3:[object Element]:null")
    );
    assert_eq!(
        host.html_collection_tag_name_items_calls,
        vec![all_collection.clone(), all_collection.clone()]
    );
    assert_eq!(
        host.html_collection_tag_name_named_item_calls,
        vec![
            (all_collection.clone(), "second".to_string()),
            (all_collection, "missing".to_string()),
        ]
    );
}

#[test]
fn runtime_resolves_get_elements_by_tag_name_ns_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("icon", ElementHandle::new(1), "");
    host.seed_element("rect", ElementHandle::new(2), "");
    host.seed_element("dot", ElementHandle::new(3), "");
    host.seed_element("out", ElementHandle::new(4), "");
    let document_collection = HtmlCollectionTarget::ByTagNameNs {
        scope: HtmlCollectionScope::Document,
        namespace_uri: "http://www.w3.org/2000/svg".to_string(),
        local_name: "*".to_string(),
    };
    let scoped_collection = HtmlCollectionTarget::ByTagNameNs {
        scope: HtmlCollectionScope::Element(ElementHandle::new(1)),
        namespace_uri: "http://www.w3.org/2000/svg".to_string(),
        local_name: "rect".to_string(),
    };
    host.seed_html_collection_tag_name_ns_items(
        document_collection.clone(),
        vec![
            ElementHandle::new(1),
            ElementHandle::new(2),
            ElementHandle::new(3),
        ],
    );
    host.seed_html_collection_tag_name_ns_items(
        scoped_collection.clone(),
        vec![ElementHandle::new(2)],
    );
    host.seed_html_collection_tag_name_ns_named_item(
        document_collection.clone(),
        "dot",
        Some(ElementHandle::new(3)),
    );
    host.seed_html_collection_tag_name_ns_named_item(document_collection.clone(), "missing", None);

    runtime
        .eval_program(
            "const all = document.getElementsByTagNameNS('http://www.w3.org/2000/svg', '*'); const scoped = document.getElementById('icon').getElementsByTagNameNS('http://www.w3.org/2000/svg', 'rect'); const dot = all.namedItem('dot'); document.getElementById('out').textContent = String(all.length) + ':' + String(scoped.length) + ':' + String(dot) + ':' + String(all.namedItem('missing'));",
            "inline-script",
            &mut host,
        )
        .expect("getElementsByTagNameNS should resolve");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(4))
            .map(String::as_str),
        Some("3:1:[object Element]:null")
    );
    assert_eq!(
        host.html_collection_tag_name_ns_items_calls,
        vec![document_collection.clone(), scoped_collection]
    );
    assert_eq!(
        host.html_collection_tag_name_ns_named_item_calls,
        vec![
            (document_collection.clone(), "dot".to_string()),
            (document_collection, "missing".to_string()),
        ]
    );
}

#[test]
fn runtime_resolves_html_collection_named_item_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("root", ElementHandle::new(1), "root");
    host.seed_element("first", ElementHandle::new(2), "first");
    host.seed_element("out", ElementHandle::new(3), "");
    host.seed_html_collection_named_item(
        ElementHandle::new(1),
        "alpha",
        Some(ElementHandle::new(2)),
    );
    host.seed_html_collection_named_item(ElementHandle::new(1), "missing", None);

    runtime
        .eval_program(
            "const children = document.getElementById('root').children; document.getElementById('out').textContent = children.namedItem('alpha').textContent + ':' + String(children.namedItem('missing'));",
            "inline-script",
            &mut host,
        )
        .expect("HTMLCollection namedItem should resolve");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(3))
            .map(String::as_str),
        Some("first:null")
    );
    assert_eq!(
        host.html_collection_named_item_calls,
        vec![
            (ElementHandle::new(1), "alpha".to_string()),
            (ElementHandle::new(1), "missing".to_string()),
        ]
    );
}

#[test]
fn runtime_resolves_document_get_elements_by_name_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("root", ElementHandle::new(1), "root");
    host.seed_element("first", ElementHandle::new(2), "First");
    host.seed_element("second", ElementHandle::new(3), "Second");
    host.seed_element("out", ElementHandle::new(4), "");
    host.seed_document_get_elements_by_name(
        "alpha",
        vec![ElementHandle::new(2), ElementHandle::new(3)],
    );

    runtime
        .eval_program(
            "const nodes = document.getElementsByName('alpha'); const first = nodes.item(0); const before = nodes.length; document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(before) + ':' + String(nodes.length) + ':' + first.textContent + ':' + String(nodes.item(1));",
            "inline-script",
            &mut host,
        )
        .expect("document.getElementsByName should resolve");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(4))
            .map(String::as_str),
        Some("2:2:First:[object Element]")
    );
    assert_eq!(
        host.document_get_elements_by_name_calls,
        vec![
            "alpha".to_string(),
            "alpha".to_string(),
            "alpha".to_string(),
            "alpha".to_string(),
        ]
    );
}

#[test]
fn runtime_reports_get_elements_by_name_on_elements_explicitly() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("root", ElementHandle::new(1), "");

    let error = runtime
        .eval_program(
            "document.getElementById('root').getElementsByName('alpha');",
            "inline-script",
            &mut host,
        )
        .expect_err("element.getElementsByName should fail explicitly");

    assert!(
        error
            .to_string()
            .contains("unsupported Element method: getElementsByName")
    );
}

#[test]
fn runtime_resolves_element_matches_for_current_element_only() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("root", ElementHandle::new(1), "");
    host.seed_element("child", ElementHandle::new(2), "");
    host.seed_element("out", ElementHandle::new(3), "");
    host.seed_element_matches(ElementHandle::new(1), ".primary", true);
    host.seed_element_matches(ElementHandle::new(1), ".child", false);
    host.seed_element_matches(ElementHandle::new(2), ".child", true);

    runtime
        .eval_program(
            "const root = document.getElementById('root'); const child = document.getElementById('child'); document.getElementById('out').textContent = String(root.matches('.primary')) + ':' + String(root.matches('.child')) + ':' + String(child.matches('.child'));",
            "inline-script",
            &mut host,
        )
        .expect("matches should resolve through host bindings");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(3))
            .map(String::as_str),
        Some("true:false:true")
    );
    assert_eq!(
        host.element_matches_calls,
        vec![
            (ElementHandle::new(1), ".primary".to_string()),
            (ElementHandle::new(1), ".child".to_string()),
            (ElementHandle::new(2), ".child".to_string()),
        ]
    );
}

#[test]
fn runtime_resolves_element_closest_with_self_and_ancestor_matches() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("root", ElementHandle::new(1), "ROOTSECTIONCHILD");
    host.seed_element("section", ElementHandle::new(2), "SECTIONCHILD");
    host.seed_element("child", ElementHandle::new(3), "CHILD");
    host.seed_element("out", ElementHandle::new(4), "");
    host.seed_element_closest(
        ElementHandle::new(1),
        ".primary",
        Some(ElementHandle::new(1)),
    );
    host.seed_element_closest(ElementHandle::new(3), ".child", Some(ElementHandle::new(3)));
    host.seed_element_closest(
        ElementHandle::new(3),
        "#section",
        Some(ElementHandle::new(2)),
    );
    host.seed_element_closest(ElementHandle::new(3), ".missing", None);

    runtime
        .eval_program(
            "const root = document.getElementById('root'); const child = document.getElementById('child'); document.getElementById('out').textContent = root.closest('.primary').textContent + ':' + child.closest('.child').textContent + ':' + child.closest('#section').textContent + ':' + String(child.closest('.missing'));",
            "inline-script",
            &mut host,
        )
        .expect("closest should resolve through host bindings");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(4))
            .map(String::as_str),
        Some("ROOTSECTIONCHILD:CHILD:SECTIONCHILD:null")
    );
    assert_eq!(
        host.element_closest_calls,
        vec![
            (ElementHandle::new(1), ".primary".to_string()),
            (ElementHandle::new(3), ".child".to_string()),
            (ElementHandle::new(3), "#section".to_string()),
            (ElementHandle::new(3), ".missing".to_string()),
        ]
    );
}

#[test]
fn runtime_reports_missing_element_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program(
            "document.getElementById('missing').textContent = 'Hello';",
            "inline-script",
            &mut host,
        )
        .expect_err("missing elements should fail");

    assert!(
        error
            .to_string()
            .contains("document.getElementById(\"missing\") returned no element")
    );
}

#[test]
fn runtime_supports_node_list_for_each() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("first", ElementHandle::new(1), "One");
    host.seed_element("second", ElementHandle::new(2), "Two");
    host.seed_element("out", ElementHandle::new(3), "");
    host.seed_document_query_selector_all(
        ".item",
        vec![ElementHandle::new(1), ElementHandle::new(2)],
    );

    runtime
        .eval_program(
            "const nodes = document.querySelectorAll('.item'); nodes.forEach((item, index, list) => { document.getElementById('out').textContent += String(index) + ':' + item.textContent + ':' + String(list.length) + ';'; }, null);",
            "inline-script",
            &mut host,
        )
        .expect("NodeList.forEach should dispatch through the script runtime");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(3))
            .map(String::as_str),
        Some("0:One:2;1:Two:2;")
    );
    assert_eq!(
        host.document_query_selector_all_calls,
        vec![".item".to_string()]
    );
}

#[test]
fn runtime_supports_html_collection_for_each() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("root", ElementHandle::new(1), "");
    host.seed_element("first", ElementHandle::new(2), "One");
    host.seed_element("second", ElementHandle::new(3), "Two");
    host.seed_element("out", ElementHandle::new(4), "");
    host.seed_element_children(
        ElementHandle::new(1),
        vec![ElementHandle::new(2), ElementHandle::new(3)],
    );

    runtime
        .eval_program(
            "const children = document.getElementById('root').children; children.forEach((child, index, list) => { document.getElementById('out').textContent += String(index) + ':' + child.textContent + ':' + String(list.length) + ';'; });",
            "inline-script",
            &mut host,
        )
        .expect("HTMLCollection.forEach should dispatch through the script runtime");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(4))
            .map(String::as_str),
        Some("0:One:2;1:Two:2;")
    );
    assert_eq!(
        host.element_children_calls,
        vec![
            ElementHandle::new(1),
            ElementHandle::new(1),
            ElementHandle::new(1),
        ]
    );
}

#[test]
fn runtime_supports_collection_iterator_helpers() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("root", ElementHandle::new(1), "");
    host.seed_element("first", ElementHandle::new(2), "One");
    host.seed_element("second", ElementHandle::new(3), "Two");
    host.seed_element("out", ElementHandle::new(4), "");
    host.seed_document_query_selector_all(
        ".item",
        vec![ElementHandle::new(2), ElementHandle::new(3)],
    );
    host.seed_element_children(
        ElementHandle::new(1),
        vec![ElementHandle::new(2), ElementHandle::new(3)],
    );

    runtime
        .eval_program(
            "const nodes = document.querySelectorAll('.item'); const nodeValues = nodes.values(); const nodeKeys = nodes.keys(); const children = document.getElementById('root').children; const childValues = children.values(); const childKeys = children.keys(); const firstNode = nodeValues.next(); const secondNode = nodeValues.next(); const thirdNode = nodeValues.next(); const firstKey = nodeKeys.next(); const secondKey = nodeKeys.next(); const thirdKey = nodeKeys.next(); const firstChild = childValues.next(); const secondChild = childValues.next(); const thirdChild = childValues.next(); const childFirstKey = childKeys.next(); const childSecondKey = childKeys.next(); const childThirdKey = childKeys.next(); document.getElementById('out').textContent = firstNode.value.textContent + ':' + String(firstNode.done) + ':' + secondNode.value.textContent + ':' + String(secondNode.done) + ':' + String(thirdNode.done) + ':' + String(firstKey.value) + ':' + String(secondKey.value) + ':' + String(thirdKey.done) + ':' + firstChild.value.textContent + ':' + String(firstChild.done) + ':' + secondChild.value.textContent + ':' + String(secondChild.done) + ':' + String(thirdChild.done) + ':' + String(childFirstKey.value) + ':' + String(childSecondKey.value) + ':' + String(childThirdKey.done);",
            "inline-script",
            &mut host,
        )
        .expect("collection iterator helpers should dispatch through the script runtime");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(4))
            .map(String::as_str),
        Some("One:false:Two:false:true:0:1:true:One:false:Two:false:true:0:1:true")
    );
    assert_eq!(
        host.document_query_selector_all_calls,
        vec![".item".to_string()]
    );
    assert_eq!(
        host.element_children_calls,
        vec![ElementHandle::new(1), ElementHandle::new(1)]
    );
}

#[test]
fn runtime_rejects_collection_entries_helpers() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(1), "");
    host.seed_document_query_selector_all(".item", vec![ElementHandle::new(1)]);

    let error = runtime
        .eval_program(
            "document.querySelectorAll('.item').entries();",
            "inline-script",
            &mut host,
        )
        .expect_err("entries helper should remain unsupported");

    assert!(
        error
            .to_string()
            .contains("unsupported NodeList method: entries")
    );
}
