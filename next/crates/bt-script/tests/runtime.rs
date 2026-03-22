use std::collections::BTreeMap;

use bt_script::{
    ElementHandle, HostBindings, HtmlCollectionScope, HtmlCollectionTarget, ListenerTarget,
    MediaQueryListState, NodeHandle, RadioNodeListTarget, ScreenOrientationState, ScriptFunction,
    ScriptRuntime, StorageTarget,
};

#[derive(Default)]
struct NoopHost {
    microtasks: usize,
    navigator_mime_types_calls: usize,
}

impl HostBindings for NoopHost {
    fn on_microtask_checkpoint(&mut self) -> bt_script::Result<()> {
        self.microtasks += 1;
        Ok(())
    }

    fn window_navigator_mime_types(&mut self) -> bt_script::Result<Vec<String>> {
        self.navigator_mime_types_calls += 1;
        Ok(Vec::new())
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

fn domain_from_url(url: &str) -> String {
    let Some((_, rest)) = url.split_once(':') else {
        return "null".to_string();
    };

    let Some(after_slashes) = rest.strip_prefix("//") else {
        return "null".to_string();
    };

    let authority_end = after_slashes
        .find(['/', '?', '#'])
        .unwrap_or(after_slashes.len());
    let mut authority = &after_slashes[..authority_end];
    if authority.is_empty() {
        return "null".to_string();
    }

    if let Some((_, host)) = authority.rsplit_once('@') {
        authority = host;
    }

    let host = if authority.starts_with('[') {
        let Some(end_bracket) = authority.find(']') else {
            return "null".to_string();
        };
        &authority[1..end_bracket]
    } else {
        authority
            .split_once(':')
            .map(|(host, _)| host)
            .unwrap_or(authority)
    };

    if host.is_empty() {
        "null".to_string()
    } else {
        host.to_ascii_lowercase()
    }
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
    document_style_sheets_named_item_results: BTreeMap<String, Option<ElementHandle>>,
    document_children_items_results: Vec<ElementHandle>,
    window_frames_items_results: Vec<ElementHandle>,
    node_child_nodes_items_results: BTreeMap<HtmlCollectionScope, Vec<NodeHandle>>,
    node_parent_results: BTreeMap<NodeHandle, Option<NodeHandle>>,
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
    window_frames_named_item_results: BTreeMap<String, Option<ElementHandle>>,
    table_rows_named_item_results: BTreeMap<(ElementHandle, String), Option<ElementHandle>>,
    row_cells_named_item_results: BTreeMap<(ElementHandle, String), Option<ElementHandle>>,
    document_query_selector_results: BTreeMap<String, Option<ElementHandle>>,
    document_query_selector_all_results: BTreeMap<String, Vec<ElementHandle>>,
    document_get_elements_by_name_results: BTreeMap<String, Vec<ElementHandle>>,
    document_document_element_result: Option<ElementHandle>,
    document_head_result: Option<ElementHandle>,
    document_body_result: Option<ElementHandle>,
    document_scrolling_element_result: Option<ElementHandle>,
    document_active_element_result: Option<ElementHandle>,
    document_has_child_nodes_result: bool,
    document_has_child_nodes_calls: usize,
    document_has_focus_result: bool,
    document_visibility_state_result: String,
    document_hidden_result: bool,
    document_title_result: String,
    document_location_result: String,
    document_compat_mode_result: String,
    document_character_set_result: String,
    document_content_type_result: String,
    document_design_mode_result: String,
    document_dir_result: String,
    document_base_uri_calls: usize,
    document_origin_calls: usize,
    document_domain_calls: usize,
    document_referrer_result: String,
    document_referrer_calls: usize,
    document_cookie_jar: BTreeMap<String, String>,
    document_cookie_calls: usize,
    document_set_cookie_calls: Vec<String>,
    window_name_result: String,
    window_name_calls: usize,
    set_window_name_calls: Vec<String>,
    window_history_length_result: usize,
    window_history_state_result: Option<String>,
    window_history_scroll_restoration_result: String,
    window_history_back_calls: usize,
    window_history_forward_calls: usize,
    window_history_go_calls: Vec<i64>,
    window_history_push_state_calls: Vec<(Option<String>, Option<String>)>,
    window_history_replace_state_calls: Vec<(Option<String>, Option<String>)>,
    window_open_calls: Vec<(Option<String>, Option<String>, Option<String>)>,
    window_open_error: Option<String>,
    window_close_calls: usize,
    window_close_error: Option<String>,
    window_print_calls: usize,
    window_print_error: Option<String>,
    dialog_alert_messages: Vec<String>,
    dialog_confirm_messages: Vec<String>,
    dialog_prompt_messages: Vec<String>,
    dialog_confirm_queue: Vec<bool>,
    dialog_prompt_queue: Vec<Option<String>>,
    navigator_user_agent: String,
    navigator_platform: String,
    navigator_language: String,
    navigator_languages_calls: usize,
    navigator_mime_types_calls: usize,
    navigator_cookie_enabled: bool,
    navigator_on_line: bool,
    window_scroll_calls: Vec<(String, i64, i64)>,
    window_scroll_error: Option<String>,
    window_scroll_x: i64,
    window_scroll_y: i64,
    window_device_pixel_ratio_result: f64,
    match_media_matches: BTreeMap<String, bool>,
    match_media_calls: Vec<String>,
    local_storage: BTreeMap<String, String>,
    session_storage: BTreeMap<String, String>,
    document_compat_mode_calls: usize,
    document_character_set_calls: usize,
    document_content_type_calls: usize,
    document_dir_calls: usize,
    element_base_uri_calls: Vec<ElementHandle>,
    element_origin_calls: Vec<ElementHandle>,
    element_is_content_editable_calls: Vec<ElementHandle>,
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
    document_style_sheets_named_item_calls: Vec<String>,
    document_children_items_calls: usize,
    window_frames_items_calls: usize,
    node_child_nodes_items_calls: Vec<HtmlCollectionScope>,
    document_contains_calls: Vec<NodeHandle>,
    node_contains_calls: Vec<(NodeHandle, NodeHandle)>,
    node_is_equal_node_results: BTreeMap<(NodeHandle, NodeHandle), bool>,
    node_is_equal_node_calls: Vec<(NodeHandle, NodeHandle)>,
    template_content_is_equal_node_results: BTreeMap<(ElementHandle, ElementHandle), bool>,
    template_content_is_equal_node_calls: Vec<(ElementHandle, ElementHandle)>,
    node_has_child_nodes_results: BTreeMap<NodeHandle, bool>,
    node_has_child_nodes_calls: Vec<NodeHandle>,
    node_replace_with_calls: Vec<(NodeHandle, Vec<NodeHandle>)>,
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
    window_frames_named_item_calls: Vec<String>,
    table_rows_named_item_calls: Vec<(ElementHandle, String)>,
    row_cells_named_item_calls: Vec<(ElementHandle, String)>,
    document_query_selector_calls: Vec<String>,
    document_query_selector_all_calls: Vec<String>,
    document_get_elements_by_name_calls: Vec<String>,
    document_document_element_calls: usize,
    document_head_calls: usize,
    document_body_calls: usize,
    document_scrolling_element_calls: usize,
    document_active_element_calls: usize,
    document_has_focus_calls: usize,
    document_visibility_state_calls: usize,
    document_hidden_calls: usize,
    document_title_calls: usize,
    document_set_title_calls: Vec<String>,
    document_design_mode_calls: usize,
    document_set_design_mode_calls: Vec<String>,
    document_location_calls: usize,
    document_set_location_calls: Vec<String>,
    document_location_assign_calls: Vec<String>,
    document_location_replace_calls: Vec<String>,
    document_location_reload_calls: Vec<String>,
    document_set_dir_calls: Vec<String>,
    window_history_length_calls: usize,
    window_history_state_calls: usize,
    window_history_scroll_restoration_calls: usize,
    window_history_scroll_restoration_set_calls: Vec<String>,
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

    fn seed_document_style_sheets_named_item(
        &mut self,
        name: impl Into<String>,
        result: Option<ElementHandle>,
    ) {
        self.document_style_sheets_named_item_results
            .insert(name.into(), result);
    }

    fn seed_local_storage(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.local_storage.insert(key.into(), value.into());
    }

    fn seed_session_storage(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.session_storage.insert(key.into(), value.into());
    }

    fn seed_document_children_items(&mut self, result: Vec<ElementHandle>) {
        self.document_children_items_results = result;
    }

    fn seed_window_frames_items(&mut self, result: Vec<ElementHandle>) {
        self.window_frames_items_results = result;
    }

    fn seed_node_child_nodes_items(&mut self, scope: HtmlCollectionScope, result: Vec<NodeHandle>) {
        self.node_child_nodes_items_results.insert(scope, result);
    }

    fn seed_node_parent(&mut self, node: NodeHandle, result: Option<NodeHandle>) {
        self.node_parent_results.insert(node, result);
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

    fn seed_window_frames_named_item(
        &mut self,
        name: impl Into<String>,
        result: Option<ElementHandle>,
    ) {
        self.window_frames_named_item_results
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

    fn seed_document_scrolling_element(&mut self, result: Option<ElementHandle>) {
        self.document_scrolling_element_result = result;
    }

    fn seed_document_active_element(&mut self, result: Option<ElementHandle>) {
        self.document_active_element_result = result;
    }

    fn seed_document_has_focus(&mut self, result: bool) {
        self.document_has_focus_result = result;
    }

    fn seed_document_visibility_state(&mut self, result: impl Into<String>) {
        self.document_visibility_state_result = result.into();
    }

    fn seed_document_hidden(&mut self, result: bool) {
        self.document_hidden_result = result;
    }

    fn seed_document_title(&mut self, result: impl Into<String>) {
        self.document_title_result = result.into();
    }

    fn seed_document_location(&mut self, result: impl Into<String>) {
        self.document_location_result = result.into();
    }

    fn seed_document_compat_mode(&mut self, result: impl Into<String>) {
        self.document_compat_mode_result = result.into();
    }

    fn seed_document_referrer(&mut self, result: impl Into<String>) {
        self.document_referrer_result = result.into();
    }

    fn seed_window_name(&mut self, result: impl Into<String>) {
        self.window_name_result = result.into();
    }

    fn seed_window_open_error(&mut self, message: impl Into<String>) {
        self.window_open_error = Some(message.into());
    }

    fn seed_window_close_error(&mut self, message: impl Into<String>) {
        self.window_close_error = Some(message.into());
    }

    fn seed_window_print_error(&mut self, message: impl Into<String>) {
        self.window_print_error = Some(message.into());
    }

    fn seed_window_scroll_error(&mut self, message: impl Into<String>) {
        self.window_scroll_error = Some(message.into());
    }

    fn seed_window_device_pixel_ratio(&mut self, result: f64) {
        self.window_device_pixel_ratio_result = result;
    }

    fn seed_navigator(
        &mut self,
        user_agent: impl Into<String>,
        platform: impl Into<String>,
        language: impl Into<String>,
        cookie_enabled: bool,
        on_line: bool,
    ) {
        self.navigator_user_agent = user_agent.into();
        self.navigator_platform = platform.into();
        self.navigator_language = language.into();
        self.navigator_cookie_enabled = cookie_enabled;
        self.navigator_on_line = on_line;
    }

    fn seed_match_media_result(&mut self, query: impl Into<String>, matches: bool) {
        self.match_media_matches.insert(query.into(), matches);
    }

    fn seed_document_character_set(&mut self, result: impl Into<String>) {
        self.document_character_set_result = result.into();
    }

    fn seed_document_content_type(&mut self, result: impl Into<String>) {
        self.document_content_type_result = result.into();
    }

    fn seed_document_design_mode(&mut self, result: impl Into<String>) {
        self.document_design_mode_result = result.into();
    }

    fn seed_document_dir(&mut self, result: impl Into<String>) {
        self.document_dir_result = result.into();
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

    fn document_scrolling_element(&mut self) -> bt_script::Result<Option<ElementHandle>> {
        self.document_scrolling_element_calls += 1;
        Ok(self.document_scrolling_element_result)
    }

    fn document_active_element(&mut self) -> bt_script::Result<Option<ElementHandle>> {
        self.document_active_element_calls += 1;
        Ok(self.document_active_element_result)
    }

    fn document_has_focus(&mut self) -> bt_script::Result<bool> {
        self.document_has_focus_calls += 1;
        Ok(self.document_has_focus_result)
    }

    fn document_visibility_state(&mut self) -> bt_script::Result<String> {
        self.document_visibility_state_calls += 1;
        Ok(self.document_visibility_state_result.clone())
    }

    fn document_hidden(&mut self) -> bt_script::Result<bool> {
        self.document_hidden_calls += 1;
        Ok(self.document_hidden_result)
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

    fn document_domain(&mut self) -> bt_script::Result<String> {
        self.document_domain_calls += 1;
        Ok(domain_from_url(&self.document_location_result))
    }

    fn document_referrer(&mut self) -> bt_script::Result<String> {
        self.document_referrer_calls += 1;
        Ok(self.document_referrer_result.clone())
    }

    fn document_cookie(&mut self) -> bt_script::Result<String> {
        self.document_cookie_calls += 1;
        Ok(self
            .document_cookie_jar
            .iter()
            .map(|(name, value)| format!("{name}={value}"))
            .collect::<Vec<_>>()
            .join("; "))
    }

    fn document_set_cookie(&mut self, value: &str) -> bt_script::Result<()> {
        self.document_set_cookie_calls.push(value.to_string());
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(bt_script::ScriptError::new(
                "document.cookie requires a non-empty cookie string",
            ));
        }

        let pair = trimmed
            .split_once(';')
            .map(|(pair, _)| pair)
            .unwrap_or(trimmed);
        let Some((name, cookie_value)) = pair.split_once('=') else {
            return Err(bt_script::ScriptError::new(
                "document.cookie requires `name=value`",
            ));
        };

        let name = name.trim();
        if name.is_empty() {
            return Err(bt_script::ScriptError::new(
                "document.cookie requires a non-empty cookie name",
            ));
        }

        self.document_cookie_jar
            .insert(name.to_string(), cookie_value.trim_start().to_string());
        Ok(())
    }

    fn window_name(&mut self) -> bt_script::Result<String> {
        self.window_name_calls += 1;
        Ok(self.window_name_result.clone())
    }

    fn window_open(
        &mut self,
        url: Option<&str>,
        target: Option<&str>,
        features: Option<&str>,
    ) -> bt_script::Result<()> {
        self.window_open_calls.push((
            url.map(str::to_string),
            target.map(str::to_string),
            features.map(str::to_string),
        ));
        if let Some(message) = &self.window_open_error {
            return Err(bt_script::ScriptError::new(message.clone()));
        }
        Ok(())
    }

    fn window_close(&mut self) -> bt_script::Result<()> {
        self.window_close_calls += 1;
        if let Some(message) = &self.window_close_error {
            return Err(bt_script::ScriptError::new(message.clone()));
        }
        Ok(())
    }

    fn window_print(&mut self) -> bt_script::Result<()> {
        self.window_print_calls += 1;
        if let Some(message) = &self.window_print_error {
            return Err(bt_script::ScriptError::new(message.clone()));
        }
        Ok(())
    }

    fn window_alert(&mut self, message: &str) -> bt_script::Result<()> {
        self.dialog_alert_messages.push(message.to_string());
        Ok(())
    }

    fn window_confirm(&mut self, message: &str) -> bt_script::Result<bool> {
        self.dialog_confirm_messages.push(message.to_string());
        if self.dialog_confirm_queue.is_empty() {
            return Err(bt_script::ScriptError::new(
                "confirm() requires a queued response",
            ));
        }

        Ok(self.dialog_confirm_queue.remove(0))
    }

    fn window_prompt(
        &mut self,
        message: &str,
        _default_text: Option<&str>,
    ) -> bt_script::Result<Option<String>> {
        self.dialog_prompt_messages.push(message.to_string());
        if self.dialog_prompt_queue.is_empty() {
            return Err(bt_script::ScriptError::new(
                "prompt() requires a queued response",
            ));
        }

        Ok(self.dialog_prompt_queue.remove(0))
    }

    fn html_collection_window_frames_items(&mut self) -> bt_script::Result<Vec<ElementHandle>> {
        self.window_frames_items_calls += 1;
        Ok(self.window_frames_items_results.clone())
    }

    fn html_collection_window_frames_named_item(
        &mut self,
        name: &str,
    ) -> bt_script::Result<Option<ElementHandle>> {
        self.window_frames_named_item_calls.push(name.to_string());
        Ok(self
            .window_frames_named_item_results
            .get(name)
            .copied()
            .unwrap_or(None))
    }

    fn window_navigator_user_agent(&mut self) -> bt_script::Result<String> {
        Ok(self.navigator_user_agent.clone())
    }

    fn window_navigator_app_code_name(&mut self) -> bt_script::Result<String> {
        Ok("browser-tester-next".to_string())
    }

    fn window_navigator_app_name(&mut self) -> bt_script::Result<String> {
        Ok("browser-tester-next".to_string())
    }

    fn window_navigator_app_version(&mut self) -> bt_script::Result<String> {
        Ok("browser-tester-next".to_string())
    }

    fn window_navigator_product(&mut self) -> bt_script::Result<String> {
        Ok("browser-tester-next".to_string())
    }

    fn window_navigator_product_sub(&mut self) -> bt_script::Result<String> {
        Ok("browser-tester-next".to_string())
    }

    fn window_navigator_platform(&mut self) -> bt_script::Result<String> {
        Ok(self.navigator_platform.clone())
    }

    fn window_navigator_language(&mut self) -> bt_script::Result<String> {
        Ok(self.navigator_language.clone())
    }

    fn window_navigator_oscpu(&mut self) -> bt_script::Result<String> {
        Ok("unknown".to_string())
    }

    fn window_navigator_user_language(&mut self) -> bt_script::Result<String> {
        Ok(self.navigator_language.clone())
    }

    fn window_navigator_browser_language(&mut self) -> bt_script::Result<String> {
        Ok(self.navigator_language.clone())
    }

    fn window_navigator_system_language(&mut self) -> bt_script::Result<String> {
        Ok(self.navigator_language.clone())
    }

    fn window_navigator_languages(&mut self) -> bt_script::Result<Vec<String>> {
        self.navigator_languages_calls += 1;
        Ok(vec![self.navigator_language.clone()])
    }

    fn window_navigator_mime_types(&mut self) -> bt_script::Result<Vec<String>> {
        self.navigator_mime_types_calls += 1;
        Ok(Vec::new())
    }

    fn window_navigator_cookie_enabled(&mut self) -> bt_script::Result<bool> {
        Ok(self.navigator_cookie_enabled)
    }

    fn window_navigator_on_line(&mut self) -> bt_script::Result<bool> {
        Ok(self.navigator_on_line)
    }

    fn window_navigator_webdriver(&mut self) -> bt_script::Result<bool> {
        Ok(false)
    }

    fn window_navigator_vendor(&mut self) -> bt_script::Result<String> {
        Ok("browser-tester-next".to_string())
    }

    fn window_navigator_vendor_sub(&mut self) -> bt_script::Result<String> {
        Ok("browser-tester-next".to_string())
    }

    fn window_navigator_pdf_viewer_enabled(&mut self) -> bt_script::Result<bool> {
        Ok(false)
    }

    fn window_navigator_do_not_track(&mut self) -> bt_script::Result<String> {
        Ok("unspecified".to_string())
    }

    fn window_navigator_java_enabled(&mut self) -> bt_script::Result<bool> {
        Ok(false)
    }

    fn window_navigator_hardware_concurrency(&mut self) -> bt_script::Result<i64> {
        Ok(8)
    }

    fn window_navigator_max_touch_points(&mut self) -> bt_script::Result<i64> {
        Ok(0)
    }

    fn window_history_length(&mut self) -> bt_script::Result<usize> {
        self.window_history_length_calls += 1;
        Ok(self.window_history_length_result)
    }

    fn window_history_state(&mut self) -> bt_script::Result<Option<String>> {
        self.window_history_state_calls += 1;
        Ok(self.window_history_state_result.clone())
    }

    fn window_history_scroll_restoration(&mut self) -> bt_script::Result<String> {
        self.window_history_scroll_restoration_calls += 1;
        Ok(self.window_history_scroll_restoration_result.clone())
    }

    fn set_window_history_scroll_restoration(&mut self, value: &str) -> bt_script::Result<()> {
        match value {
            "auto" | "manual" => {
                self.window_history_scroll_restoration_set_calls
                    .push(value.to_string());
                self.window_history_scroll_restoration_result = value.to_string();
                Ok(())
            }
            other => Err(bt_script::ScriptError::new(format!(
                "unsupported history scroll restoration value: {other}"
            ))),
        }
    }

    fn window_history_push_state(
        &mut self,
        state: Option<&str>,
        url: Option<&str>,
    ) -> bt_script::Result<()> {
        self.window_history_push_state_calls
            .push((state.map(str::to_string), url.map(str::to_string)));
        self.window_history_length_result += 1;
        self.window_history_state_result = state.map(str::to_string);
        if let Some(url) = url {
            self.document_location_result = url.to_string();
        }
        Ok(())
    }

    fn window_history_replace_state(
        &mut self,
        state: Option<&str>,
        url: Option<&str>,
    ) -> bt_script::Result<()> {
        self.window_history_replace_state_calls
            .push((state.map(str::to_string), url.map(str::to_string)));
        self.window_history_state_result = state.map(str::to_string);
        if let Some(url) = url {
            self.document_location_result = url.to_string();
        }
        Ok(())
    }

    fn window_history_back(&mut self) -> bt_script::Result<()> {
        self.window_history_back_calls += 1;
        Ok(())
    }

    fn window_history_forward(&mut self) -> bt_script::Result<()> {
        self.window_history_forward_calls += 1;
        Ok(())
    }

    fn window_history_go(&mut self, delta: i64) -> bt_script::Result<()> {
        self.window_history_go_calls.push(delta);
        Ok(())
    }

    fn window_scroll_to(&mut self, x: i64, y: i64) -> bt_script::Result<()> {
        self.window_scroll_calls
            .push(("scrollTo".to_string(), x, y));
        if let Some(message) = &self.window_scroll_error {
            return Err(bt_script::ScriptError::new(message.clone()));
        }
        self.window_scroll_x = x;
        self.window_scroll_y = y;
        Ok(())
    }

    fn window_scroll_by(&mut self, x: i64, y: i64) -> bt_script::Result<()> {
        self.window_scroll_calls
            .push(("scrollBy".to_string(), x, y));
        if let Some(message) = &self.window_scroll_error {
            return Err(bt_script::ScriptError::new(message.clone()));
        }
        self.window_scroll_x += x;
        self.window_scroll_y += y;
        Ok(())
    }

    fn window_scroll_x(&mut self) -> bt_script::Result<i64> {
        Ok(self.window_scroll_x)
    }

    fn window_scroll_y(&mut self) -> bt_script::Result<i64> {
        Ok(self.window_scroll_y)
    }

    fn window_page_x_offset(&mut self) -> bt_script::Result<i64> {
        Ok(self.window_scroll_x)
    }

    fn window_page_y_offset(&mut self) -> bt_script::Result<i64> {
        Ok(self.window_scroll_y)
    }

    fn window_device_pixel_ratio(&mut self) -> bt_script::Result<f64> {
        Ok(self.window_device_pixel_ratio_result)
    }

    fn window_inner_width(&mut self) -> bt_script::Result<i64> {
        Ok(1024)
    }

    fn window_inner_height(&mut self) -> bt_script::Result<i64> {
        Ok(768)
    }

    fn window_outer_width(&mut self) -> bt_script::Result<i64> {
        Ok(1024)
    }

    fn window_outer_height(&mut self) -> bt_script::Result<i64> {
        Ok(768)
    }

    fn window_screen_x(&mut self) -> bt_script::Result<i64> {
        Ok(0)
    }

    fn window_screen_y(&mut self) -> bt_script::Result<i64> {
        Ok(0)
    }

    fn window_screen_left(&mut self) -> bt_script::Result<i64> {
        Ok(0)
    }

    fn window_screen_top(&mut self) -> bt_script::Result<i64> {
        Ok(0)
    }

    fn window_screen_width(&mut self) -> bt_script::Result<i64> {
        Ok(1024)
    }

    fn window_screen_height(&mut self) -> bt_script::Result<i64> {
        Ok(768)
    }

    fn window_screen_avail_width(&mut self) -> bt_script::Result<i64> {
        Ok(1024)
    }

    fn window_screen_avail_height(&mut self) -> bt_script::Result<i64> {
        Ok(768)
    }

    fn window_screen_avail_left(&mut self) -> bt_script::Result<i64> {
        Ok(0)
    }

    fn window_screen_avail_top(&mut self) -> bt_script::Result<i64> {
        Ok(0)
    }

    fn window_screen_color_depth(&mut self) -> bt_script::Result<i64> {
        Ok(24)
    }

    fn window_screen_pixel_depth(&mut self) -> bt_script::Result<i64> {
        Ok(24)
    }

    fn window_screen_orientation(&mut self) -> bt_script::Result<ScreenOrientationState> {
        Ok(ScreenOrientationState::new("landscape-primary", 0))
    }

    fn match_media(&mut self, query: &str) -> bt_script::Result<MediaQueryListState> {
        self.match_media_calls.push(query.to_string());
        let matches = self
            .match_media_matches
            .get(query)
            .copied()
            .ok_or_else(|| {
                bt_script::ScriptError::new(format!("no matchMedia mock configured for `{query}`"))
            })?;
        Ok(MediaQueryListState::new(query, matches))
    }

    fn set_window_name(&mut self, value: &str) -> bt_script::Result<()> {
        self.set_window_name_calls.push(value.to_string());
        self.window_name_result = value.to_string();
        Ok(())
    }

    fn storage_length(&mut self, target: StorageTarget) -> bt_script::Result<usize> {
        Ok(match target {
            StorageTarget::Local => self.local_storage.len(),
            StorageTarget::Session => self.session_storage.len(),
        })
    }

    fn storage_get_item(
        &mut self,
        target: StorageTarget,
        key: &str,
    ) -> bt_script::Result<Option<String>> {
        Ok(match target {
            StorageTarget::Local => self.local_storage.get(key).cloned(),
            StorageTarget::Session => self.session_storage.get(key).cloned(),
        })
    }

    fn storage_set_item(
        &mut self,
        target: StorageTarget,
        key: &str,
        value: &str,
    ) -> bt_script::Result<()> {
        match target {
            StorageTarget::Local => {
                self.local_storage
                    .insert(key.to_string(), value.to_string());
            }
            StorageTarget::Session => {
                self.session_storage
                    .insert(key.to_string(), value.to_string());
            }
        }
        Ok(())
    }

    fn storage_remove_item(&mut self, target: StorageTarget, key: &str) -> bt_script::Result<()> {
        match target {
            StorageTarget::Local => {
                self.local_storage.remove(key);
            }
            StorageTarget::Session => {
                self.session_storage.remove(key);
            }
        }
        Ok(())
    }

    fn storage_clear(&mut self, target: StorageTarget) -> bt_script::Result<()> {
        match target {
            StorageTarget::Local => self.local_storage.clear(),
            StorageTarget::Session => self.session_storage.clear(),
        }
        Ok(())
    }

    fn storage_key(
        &mut self,
        target: StorageTarget,
        index: usize,
    ) -> bt_script::Result<Option<String>> {
        Ok(match target {
            StorageTarget::Local => self.local_storage.keys().nth(index).cloned(),
            StorageTarget::Session => self.session_storage.keys().nth(index).cloned(),
        })
    }

    fn document_compat_mode(&mut self) -> bt_script::Result<String> {
        self.document_compat_mode_calls += 1;
        Ok(self.document_compat_mode_result.clone())
    }

    fn document_character_set(&mut self) -> bt_script::Result<String> {
        self.document_character_set_calls += 1;
        Ok(self.document_character_set_result.clone())
    }

    fn document_content_type(&mut self) -> bt_script::Result<String> {
        self.document_content_type_calls += 1;
        Ok(self.document_content_type_result.clone())
    }

    fn document_design_mode(&mut self) -> bt_script::Result<String> {
        self.document_design_mode_calls += 1;
        Ok(self.document_design_mode_result.clone())
    }

    fn document_set_design_mode(&mut self, value: &str) -> bt_script::Result<()> {
        self.document_set_design_mode_calls.push(value.to_string());
        if value.eq_ignore_ascii_case("on") {
            self.document_design_mode_result = "on".to_string();
            Ok(())
        } else if value.eq_ignore_ascii_case("off") {
            self.document_design_mode_result = "off".to_string();
            Ok(())
        } else {
            Err(bt_script::ScriptError::new(format!(
                "unsupported document designMode value: {value}"
            )))
        }
    }

    fn document_dir(&mut self) -> bt_script::Result<String> {
        self.document_dir_calls += 1;
        Ok(self.document_dir_result.clone())
    }

    fn document_set_location(&mut self, value: &str) -> bt_script::Result<()> {
        self.document_set_location_calls.push(value.to_string());
        self.document_location_result = value.to_string();
        Ok(())
    }

    fn document_location_assign(&mut self, value: &str) -> bt_script::Result<()> {
        self.document_location_assign_calls.push(value.to_string());
        self.document_location_result = value.to_string();
        Ok(())
    }

    fn document_location_replace(&mut self, value: &str) -> bt_script::Result<()> {
        self.document_location_replace_calls.push(value.to_string());
        self.document_location_result = value.to_string();
        Ok(())
    }

    fn document_location_reload(&mut self) -> bt_script::Result<()> {
        self.document_location_reload_calls
            .push(self.document_location_result.clone());
        Ok(())
    }

    fn document_set_dir(&mut self, value: &str) -> bt_script::Result<()> {
        self.document_set_dir_calls.push(value.to_string());
        self.document_dir_result = value.to_string();
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

    fn element_is_content_editable(&mut self, element: ElementHandle) -> bt_script::Result<bool> {
        self.element_is_content_editable_calls.push(element);
        let value = self
            .attributes
            .get(&(element, "contenteditable".to_string()));
        Ok(matches!(
            value.map(|value| value.trim().to_ascii_lowercase()),
            Some(value) if value.is_empty() || value == "true" || value == "plaintext-only"
        ))
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

    fn radio_node_list_set_value(
        &mut self,
        target: &RadioNodeListTarget,
        value: &str,
    ) -> bt_script::Result<()> {
        let RadioNodeListTarget::FormElements { element, name } = target;
        let items = self.html_collection_form_elements_named_items(*element, name)?;
        let mut matched = false;

        for item in items {
            let tag_name = self.element_tag_name(item)?;
            if tag_name != "input" {
                continue;
            }

            let Some(input_type) = self.element_get_attribute(item, "type")? else {
                continue;
            };
            if !input_type.eq_ignore_ascii_case("radio") {
                continue;
            }

            let should_check = !matched && self.element_value(item)? == value;
            if should_check {
                matched = true;
            }
            self.element_set_checked(item, should_check)?;
        }

        Ok(())
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

    fn document_style_sheets_named_item(
        &mut self,
        name: &str,
    ) -> bt_script::Result<Option<ElementHandle>> {
        self.document_style_sheets_named_item_calls
            .push(name.to_string());
        Ok(self
            .document_style_sheets_named_item_results
            .get(name)
            .copied()
            .flatten())
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

    fn node_replace_with(
        &mut self,
        node: NodeHandle,
        children: Vec<NodeHandle>,
    ) -> bt_script::Result<()> {
        self.node_replace_with_calls.push((node, children));
        Ok(())
    }

    fn document_contains(&mut self, node: NodeHandle) -> bt_script::Result<bool> {
        self.document_contains_calls.push(node);
        Ok(true)
    }

    fn node_contains(&mut self, node: NodeHandle, other: NodeHandle) -> bt_script::Result<bool> {
        self.node_contains_calls.push((node, other));
        Ok(true)
    }

    fn node_is_equal_node(
        &mut self,
        node: NodeHandle,
        other: NodeHandle,
    ) -> bt_script::Result<bool> {
        self.node_is_equal_node_calls.push((node, other));
        Ok(self
            .node_is_equal_node_results
            .get(&(node, other))
            .copied()
            .unwrap_or(false))
    }

    fn template_content_is_equal_node(
        &mut self,
        fragment: ElementHandle,
        other: ElementHandle,
    ) -> bt_script::Result<bool> {
        self.template_content_is_equal_node_calls
            .push((fragment, other));
        Ok(self
            .template_content_is_equal_node_results
            .get(&(fragment, other))
            .copied()
            .unwrap_or(false))
    }

    fn document_has_child_nodes(&mut self) -> bt_script::Result<bool> {
        self.document_has_child_nodes_calls += 1;
        Ok(self.document_has_child_nodes_result)
    }

    fn node_has_child_nodes(&mut self, node: NodeHandle) -> bt_script::Result<bool> {
        self.node_has_child_nodes_calls.push(node);
        Ok(*self
            .node_has_child_nodes_results
            .get(&node)
            .unwrap_or(&false))
    }

    fn node_parent(&mut self, node: NodeHandle) -> bt_script::Result<Option<NodeHandle>> {
        self.node_parent_results
            .get(&node)
            .copied()
            .ok_or_else(|| bt_script::ScriptError::phase_not_ready("Node.parentNode"))
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
    host.seed_document_scrolling_element(Some(ElementHandle::new(1)));

    runtime
        .eval_program(
            "const html = document.documentElement; const head = document.head; const body = document.body; const scrolling = document.scrollingElement; document.getElementById('out').textContent = html.getAttribute('id') + ':' + head.getAttribute('id') + ':' + body.getAttribute('id') + ':' + scrolling.getAttribute('id');",
            "inline-script",
            &mut host,
        )
        .expect("document root/head/body/scrollingElement should resolve through host bindings");

    assert_eq!(host.document_document_element_calls, 1);
    assert_eq!(host.document_head_calls, 1);
    assert_eq!(host.document_body_calls, 1);
    assert_eq!(host.document_scrolling_element_calls, 1);
    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(4))
            .map(String::as_str),
        Some("html:head:body:html")
    );
}

#[test]
fn runtime_resolves_owner_document_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("body", ElementHandle::new(3), "");
    host.seed_element("out", ElementHandle::new(4), "");
    host.seed_document_body(Some(ElementHandle::new(3)));

    runtime
        .eval_program(
            "const body = document.body; document.getElementById('out').textContent = String(body.ownerDocument) + ':' + String(body.ownerDocument.defaultView) + ':' + String(document.ownerDocument);",
            "inline-script",
            &mut host,
        )
        .expect("ownerDocument should resolve through host bindings");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(4))
            .map(String::as_str),
        Some("[object Document]:[object Window]:null")
    );
}

#[test]
fn runtime_rejects_owner_document_assignment() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("body", ElementHandle::new(3), "");
    host.seed_document_body(Some(ElementHandle::new(3)));

    let error = runtime
        .eval_program(
            "document.body.ownerDocument = null;",
            "inline-script",
            &mut host,
        )
        .expect_err("ownerDocument should be read-only");

    assert!(error.to_string().contains("unsupported assignment target"));
    assert!(error.to_string().contains("ownerDocument"));
    assert!(error.to_string().contains("element"));
}

#[test]
fn runtime_resolves_parent_node_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("html", ElementHandle::new(1), "");
    host.seed_element("body", ElementHandle::new(3), "Text");
    host.seed_element("out", ElementHandle::new(4), "");
    host.seed_document_document_element(Some(ElementHandle::new(1)));
    host.seed_document_body(Some(ElementHandle::new(3)));
    host.seed_node_child_nodes_items(
        HtmlCollectionScope::Element(ElementHandle::new(3)),
        vec![NodeHandle::new(10)],
    );
    host.seed_node_parent(NodeHandle::new(1), Some(NodeHandle::new(0)));
    host.seed_node_parent(NodeHandle::new(3), Some(NodeHandle::new(1)));
    host.seed_node_parent(NodeHandle::new(10), Some(NodeHandle::new(3)));
    host.seed_node_type(NodeHandle::new(0), 9);
    host.seed_node_type(NodeHandle::new(1), 1);
    host.seed_node_type(NodeHandle::new(3), 1);
    host.seed_node_type(NodeHandle::new(10), 3);

    runtime
        .eval_program(
            "const body = document.body; const text = body.childNodes.item(0); document.getElementById('out').textContent = String(document.parentNode) + ':' + String(document.documentElement.parentNode) + ':' + String(document.documentElement.parentElement) + ':' + String(body.parentNode) + ':' + String(body.parentElement) + ':' + String(text.parentNode) + ':' + String(text.parentElement);",
            "inline-script",
            &mut host,
        )
        .expect("parentNode should resolve through host bindings");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(4))
            .map(String::as_str),
        Some(
            "null:[object Document]:null:[object Element]:[object Element]:[object Element]:[object Element]"
        )
    );
}

#[test]
fn runtime_rejects_parent_node_assignment() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("body", ElementHandle::new(3), "");
    host.seed_document_body(Some(ElementHandle::new(3)));

    let error = runtime
        .eval_program(
            "document.body.parentNode = null;",
            "inline-script",
            &mut host,
        )
        .expect_err("parentNode should be read-only");

    assert!(error.to_string().contains("unsupported assignment target"));
    assert!(error.to_string().contains("parentNode"));
}

#[test]
fn runtime_resolves_is_connected_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("html", ElementHandle::new(1), "");
    host.seed_element("body", ElementHandle::new(3), "");
    host.seed_element("ghost", ElementHandle::new(4), "");
    host.seed_element("out", ElementHandle::new(5), "");
    host.seed_document_document_element(Some(ElementHandle::new(1)));
    host.seed_document_body(Some(ElementHandle::new(3)));
    host.seed_node_parent(NodeHandle::new(1), Some(NodeHandle::new(0)));
    host.seed_node_parent(NodeHandle::new(3), Some(NodeHandle::new(1)));
    host.seed_node_parent(NodeHandle::new(4), None);
    host.seed_node_type(NodeHandle::new(0), 9);
    host.seed_node_type(NodeHandle::new(1), 1);
    host.seed_node_type(NodeHandle::new(3), 1);

    runtime
        .eval_program(
            "const body = document.body; const ghost = document.getElementById('ghost'); document.getElementById('out').textContent = String(document.isConnected) + ':' + String(body.isConnected) + ':' + String(ghost.isConnected);",
            "inline-script",
            &mut host,
        )
        .expect("isConnected should resolve through host bindings");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(5))
            .map(String::as_str),
        Some("true:true:false")
    );
}

#[test]
fn runtime_rejects_is_connected_assignment() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("body", ElementHandle::new(3), "");
    host.seed_document_body(Some(ElementHandle::new(3)));

    let error = runtime
        .eval_program(
            "document.body.isConnected = false;",
            "inline-script",
            &mut host,
        )
        .expect_err("isConnected should be read-only");

    assert!(error.to_string().contains("unsupported assignment target"));
    assert!(error.to_string().contains("isConnected"));
}

#[test]
fn runtime_resolves_first_and_last_element_child_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("html", ElementHandle::new(1), "");
    host.seed_element("head", ElementHandle::new(2), "");
    host.seed_element("body", ElementHandle::new(3), "");
    host.seed_element("wrapper", ElementHandle::new(4), "");
    host.seed_element("tmpl", ElementHandle::new(5), "");
    host.seed_element("first", ElementHandle::new(6), "");
    host.seed_element("last", ElementHandle::new(7), "");
    host.seed_element("tmpl-first", ElementHandle::new(8), "");
    host.seed_element("tmpl-last", ElementHandle::new(9), "");
    host.seed_element("out", ElementHandle::new(10), "");
    host.seed_element_tag_name(ElementHandle::new(5), "template");
    host.seed_attribute(ElementHandle::new(5), "id", "tmpl");
    host.seed_attribute(ElementHandle::new(10), "id", "out");
    host.seed_document_document_element(Some(ElementHandle::new(1)));
    host.seed_document_body(Some(ElementHandle::new(3)));
    host.seed_document_children_items(vec![ElementHandle::new(1)]);
    host.seed_element_children(
        ElementHandle::new(1),
        vec![ElementHandle::new(2), ElementHandle::new(3)],
    );
    host.seed_element_children(
        ElementHandle::new(3),
        vec![ElementHandle::new(4), ElementHandle::new(5)],
    );
    host.seed_element_children(
        ElementHandle::new(5),
        vec![ElementHandle::new(8), ElementHandle::new(9)],
    );

    runtime
        .eval_program(
            "const html = document.documentElement; const body = document.body; const template = document.getElementById('tmpl').content; document.getElementById('out').textContent = String(document.childElementCount) + ':' + String(document.firstElementChild) + ':' + String(document.lastElementChild) + ':' + String(html.childElementCount) + ':' + String(html.firstElementChild) + ':' + String(html.lastElementChild) + ':' + String(body.childElementCount) + ':' + String(body.firstElementChild) + ':' + String(body.lastElementChild) + ':' + String(template.childElementCount) + ':' + String(template.firstElementChild) + ':' + String(template.lastElementChild);",
            "inline-script",
            &mut host,
        )
        .expect("element child reflection should resolve through host bindings");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(10))
            .map(String::as_str),
        Some(
            "1:[object Element]:[object Element]:2:[object Element]:[object Element]:2:[object Element]:[object Element]:2:[object Element]:[object Element]"
        )
    );
}

#[test]
fn runtime_rejects_first_element_child_assignment() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("body", ElementHandle::new(3), "");
    host.seed_document_body(Some(ElementHandle::new(3)));

    let error = runtime
        .eval_program(
            "document.body.firstElementChild = null;",
            "inline-script",
            &mut host,
        )
        .expect_err("firstElementChild should be read-only");

    assert!(error.to_string().contains("unsupported assignment target"));
    assert!(error.to_string().contains("firstElementChild"));
}

#[test]
fn runtime_rejects_document_scrolling_element_assignment() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program(
            "document.scrollingElement = null;",
            "inline-script",
            &mut host,
        )
        .expect_err("document.scrollingElement should be read-only");

    assert!(error.to_string().contains("unsupported assignment target"));
    assert!(error.to_string().contains("scrollingElement"));
}

#[test]
fn runtime_resolves_document_active_element_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("first", ElementHandle::new(1), "");
    host.seed_element("out", ElementHandle::new(2), "");
    host.seed_attribute(ElementHandle::new(1), "id", "first");
    host.seed_document_active_element(Some(ElementHandle::new(1)));

    runtime
        .eval_program(
            "const active = document.activeElement; document.getElementById('out').textContent = String(active) + ':' + active.getAttribute('id');",
            "inline-script",
            &mut host,
        )
        .expect("document.activeElement should resolve through host bindings");

    assert_eq!(host.document_active_element_calls, 1);
    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(2))
            .map(String::as_str),
        Some("[object Element]:first")
    );
}

#[test]
fn runtime_resolves_document_has_focus_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(1), "");
    host.seed_document_has_focus(true);

    runtime
        .eval_program(
            "document.getElementById('out').textContent = String(document.hasFocus());",
            "inline-script",
            &mut host,
        )
        .expect("document.hasFocus should resolve through host bindings");

    assert_eq!(host.document_has_focus_calls, 1);
    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(1))
            .map(String::as_str),
        Some("true")
    );
}

#[test]
fn runtime_resolves_document_visibility_state_and_hidden_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(1), "");
    host.seed_document_visibility_state("visible");
    host.seed_document_hidden(false);

    runtime
        .eval_program(
            "document.getElementById('out').textContent = document.visibilityState + ':' + String(document.hidden);",
            "inline-script",
            &mut host,
        )
        .expect("document.visibilityState should resolve through host bindings");

    assert_eq!(host.document_visibility_state_calls, 1);
    assert_eq!(host.document_hidden_calls, 1);
    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(1))
            .map(String::as_str),
        Some("visible:false")
    );
}

#[test]
fn runtime_rejects_document_visibility_state_assignment() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program(
            "document.visibilityState = 'hidden';",
            "inline-script",
            &mut host,
        )
        .expect_err("document.visibilityState should be read-only");

    assert!(error.message().contains("unsupported assignment target"));
    assert!(error.message().contains("visibilityState"));
}

#[test]
fn runtime_rejects_document_hidden_assignment() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program("document.hidden = true;", "inline-script", &mut host)
        .expect_err("document.hidden should be read-only");

    assert!(error.message().contains("unsupported assignment target"));
    assert!(error.message().contains("hidden"));
}

#[test]
fn runtime_resolves_window_device_pixel_ratio_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(1), "");
    host.seed_window_device_pixel_ratio(2.0);

    runtime
        .eval_program(
            "document.getElementById('out').textContent = String(window.devicePixelRatio);",
            "inline-script",
            &mut host,
        )
        .expect("window.devicePixelRatio should resolve through host bindings");

    assert_eq!(host.window_device_pixel_ratio_result, 2.0);
    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(1))
            .map(String::as_str),
        Some("2")
    );
}

#[test]
fn runtime_rejects_window_device_pixel_ratio_assignment() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program("window.devicePixelRatio = 2;", "inline-script", &mut host)
        .expect_err("window.devicePixelRatio should be read-only");

    assert!(error.message().contains("unsupported assignment target"));
    assert!(error.message().contains("devicePixelRatio"));
}

#[test]
fn runtime_resolves_window_inner_width_and_inner_height_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(1), "");

    runtime
        .eval_program(
            "document.getElementById('out').textContent = String(window.innerWidth) + ':' + String(window.innerHeight);",
            "inline-script",
            &mut host,
        )
        .expect("window.innerWidth and window.innerHeight should resolve through host bindings");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(1))
            .map(String::as_str),
        Some("1024:768")
    );
}

#[test]
fn runtime_rejects_window_inner_width_assignment() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program("window.innerWidth = 2;", "inline-script", &mut host)
        .expect_err("window.innerWidth should be read-only");

    assert!(error.message().contains("unsupported assignment target"));
    assert!(error.message().contains("innerWidth"));
}

#[test]
fn runtime_resolves_window_outer_width_and_outer_height_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(1), "");

    runtime
        .eval_program(
            "document.getElementById('out').textContent = String(window.outerWidth) + ':' + String(window.outerHeight);",
            "inline-script",
            &mut host,
        )
        .expect("window.outerWidth and window.outerHeight should resolve through host bindings");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(1))
            .map(String::as_str),
        Some("1024:768")
    );
}

#[test]
fn runtime_rejects_window_outer_width_assignment() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program("window.outerWidth = 2;", "inline-script", &mut host)
        .expect_err("window.outerWidth should be read-only");

    assert!(error.message().contains("unsupported assignment target"));
    assert!(error.message().contains("outerWidth"));
}

#[test]
fn runtime_resolves_window_screen_position_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(1), "");

    runtime
        .eval_program(
            "document.getElementById('out').textContent = String(window.screenX) + ':' + String(window.screenY) + ':' + String(window.screenLeft) + ':' + String(window.screenTop);",
            "inline-script",
            &mut host,
        )
        .expect("window.screenX / screenY / screenLeft / screenTop should resolve through host bindings");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(1))
            .map(String::as_str),
        Some("0:0:0:0")
    );
}

#[test]
fn runtime_rejects_window_screen_x_assignment() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program("window.screenX = 2;", "inline-script", &mut host)
        .expect_err("window.screenX should be read-only");

    assert!(error.message().contains("unsupported assignment target"));
    assert!(error.message().contains("screenX"));
}

#[test]
fn runtime_resolves_window_screen_object_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(1), "");

    runtime
        .eval_program(
            "document.getElementById('out').textContent = String(window.screen) + ':' + String(window.screen.width) + ':' + String(window.screen.height) + ':' + String(window.screen.availWidth) + ':' + String(window.screen.availHeight) + ':' + String(window.screen.availLeft) + ':' + String(window.screen.availTop) + ':' + String(window.screen.colorDepth) + ':' + String(window.screen.pixelDepth);",
            "inline-script",
            &mut host,
        )
        .expect("window.screen should resolve through host bindings");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(1))
            .map(String::as_str),
        Some("[object Screen]:1024:768:1024:768:0:0:24:24")
    );
}

#[test]
fn runtime_resolves_window_screen_orientation_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(1), "");

    runtime
        .eval_program(
            "document.getElementById('out').textContent = String(window.screen.orientation) + ':' + window.screen.orientation.type + ':' + String(window.screen.orientation.angle);",
            "inline-script",
            &mut host,
        )
        .expect("window.screen.orientation should resolve through host bindings");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(1))
            .map(String::as_str),
        Some("[object ScreenOrientation]:landscape-primary:0")
    );
}

#[test]
fn runtime_rejects_window_screen_width_assignment() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program("window.screen.width = 2;", "inline-script", &mut host)
        .expect_err("window.screen.width should be read-only");

    assert!(error.message().contains("screen"));
    assert!(error.message().contains("width"));
}

#[test]
fn runtime_rejects_window_screen_orientation_assignment() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program(
            "window.screen.orientation.type = 'portrait-primary';",
            "inline-script",
            &mut host,
        )
        .expect_err("window.screen.orientation.type should be read-only");

    assert!(error.message().contains("screen orientation"));
    assert!(error.message().contains("type"));
}

#[test]
fn runtime_rejects_document_has_focus_with_arguments() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program("document.hasFocus(true);", "inline-script", &mut host)
        .expect_err("document.hasFocus should reject arguments");

    assert!(
        error
            .message()
            .contains("document.hasFocus() expects no arguments")
    );
}

#[test]
fn runtime_resolves_document_compat_mode_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(1), "");
    host.seed_document_compat_mode("CSS1Compat");

    runtime
        .eval_program(
            "document.getElementById('out').textContent = document.compatMode;",
            "inline-script",
            &mut host,
        )
        .expect("document.compatMode should resolve through host bindings");

    assert_eq!(host.document_compat_mode_calls, 1);
    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(1))
            .map(String::as_str),
        Some("CSS1Compat")
    );
}

#[test]
fn runtime_resolves_document_character_set_and_charset_alias_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(1), "");
    host.seed_document_character_set("UTF-8");

    runtime
        .eval_program(
            "const before = document.characterSet; const alias = document.charset; document.getElementById('out').textContent = before + ':' + alias;",
            "inline-script",
            &mut host,
        )
        .expect("document.characterSet should resolve through host bindings");

    assert_eq!(host.document_character_set_calls, 2);
    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(1))
            .map(String::as_str),
        Some("UTF-8:UTF-8")
    );
}

#[test]
fn runtime_resolves_document_content_type_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(1), "");
    host.seed_document_content_type("text/html");

    runtime
        .eval_program(
            "document.getElementById('out').textContent = document.contentType;",
            "inline-script",
            &mut host,
        )
        .expect("document.contentType should resolve through host bindings");

    assert_eq!(host.document_content_type_calls, 1);
    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(1))
            .map(String::as_str),
        Some("text/html")
    );
}

#[test]
fn runtime_resolves_document_design_mode_access_and_assignment() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(1), "");
    host.seed_document_design_mode("off");

    runtime
        .eval_program(
            "const before = document.designMode; document.designMode = 'on'; document.getElementById('out').textContent = before + ':' + document.designMode;",
            "inline-script",
            &mut host,
        )
        .expect("document.designMode should resolve through host bindings");

    assert_eq!(host.document_design_mode_calls, 2);
    assert_eq!(host.document_set_design_mode_calls, vec!["on".to_string()]);
    assert_eq!(host.document_design_mode_result, "on");
    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(1))
            .map(String::as_str),
        Some("off:on")
    );
}

#[test]
fn runtime_rejects_document_design_mode_invalid_assignment() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program("document.designMode = 'maybe';", "inline-script", &mut host)
        .expect_err("document.designMode should reject unsupported values");

    assert!(error.message().contains("designMode"));
    assert!(error.message().contains("unsupported"));
    assert_eq!(
        host.document_set_design_mode_calls,
        vec!["maybe".to_string()]
    );
}

#[test]
fn runtime_resolves_element_content_editable_access_and_assignment() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(1), "");
    host.seed_element("editable", ElementHandle::new(2), "Edit");
    host.seed_attribute(ElementHandle::new(2), "contenteditable", "TRUE");

    runtime
        .eval_program(
            "const editable = document.getElementById('editable'); const before = editable.contentEditable; const beforeState = editable.isContentEditable; editable.contentEditable = 'inherit'; const after = editable.contentEditable; const afterState = editable.isContentEditable; document.getElementById('out').textContent = before + ':' + String(beforeState) + ':' + after + ':' + String(afterState);",
            "inline-script",
            &mut host,
        )
        .expect("element.contentEditable should resolve through host bindings");

    assert_eq!(
        host.element_is_content_editable_calls,
        vec![ElementHandle::new(2), ElementHandle::new(2)]
    );
    assert!(
        !host
            .attributes
            .contains_key(&(ElementHandle::new(2), "contenteditable".to_string()))
    );
    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(1))
            .map(String::as_str),
        Some("true:true:inherit:false")
    );
}

#[test]
fn runtime_rejects_element_content_editable_invalid_assignment() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("editable", ElementHandle::new(1), "Edit");

    let error = runtime
        .eval_program(
            "document.getElementById('editable').contentEditable = 'maybe';",
            "inline-script",
            &mut host,
        )
        .expect_err("element.contentEditable should reject unsupported values");

    assert!(error.message().contains("contentEditable"));
    assert!(error.message().contains("unsupported"));
    assert!(host.element_is_content_editable_calls.is_empty());
}

#[test]
fn runtime_resolves_document_referrer_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(1), "");
    host.seed_document_referrer("https://example.test/source");

    runtime
        .eval_program(
            "document.getElementById('out').textContent = document.referrer;",
            "inline-script",
            &mut host,
        )
        .expect("document.referrer should resolve through host bindings");

    assert_eq!(host.document_referrer_calls, 1);
    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(1))
            .map(String::as_str),
        Some("https://example.test/source")
    );
}

#[test]
fn runtime_resolves_window_name_getter_and_setter_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(1), "");
    host.seed_window_name("seeded");

    runtime
        .eval_program(
            "const before = window.name; window.self.name = 'updated'; document.getElementById('out').textContent = before + ':' + window.window.name + ':' + window.parent.name + ':' + window.top.name;",
            "inline-script",
            &mut host,
        )
        .expect("window.name should resolve through host bindings");

    assert_eq!(host.window_name_calls, 4);
    assert_eq!(host.set_window_name_calls, vec!["updated".to_string()]);
    assert_eq!(host.window_name_result, "updated");
    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(1))
            .map(String::as_str),
        Some("seeded:updated:updated:updated")
    );
}

#[test]
fn runtime_rejects_window_self_assignment() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program("window.self = 'updated';", "inline-script", &mut host)
        .expect_err("window.self should be read-only");

    assert!(error.message().contains("unsupported assignment target"));
    assert!(error.message().contains("self"));
}

#[test]
fn runtime_resolves_window_closed_accessor() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(1), "");

    runtime
        .eval_program(
            "document.getElementById('out').textContent = String(window.closed) + ':' + String(window.self.closed) + ':' + String(window.window.closed) + ':' + String(window.parent.closed) + ':' + String(window.top.closed);",
            "inline-script",
            &mut host,
        )
        .expect("window.closed should resolve through host bindings");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(1))
            .map(String::as_str),
        Some("false:false:false:false:false")
    );
}

#[test]
fn runtime_rejects_window_closed_assignment() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program("window.closed = true;", "inline-script", &mut host)
        .expect_err("window.closed should be read-only");

    assert!(error.message().contains("unsupported assignment target"));
    assert!(error.message().contains("closed"));
}

#[test]
fn runtime_resolves_window_history_accessor() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(1), "");
    host.window_history_length_result = 1;
    host.window_history_scroll_restoration_result = "auto".to_string();

    runtime
        .eval_program(
            "document.getElementById('out').textContent = String(window.history) + ':' + String(window.history.length) + ':' + String(window.self.history.length) + ':' + String(window.history.state) + ':' + String(window.history.scrollRestoration);",
            "inline-script",
            &mut host,
        )
        .expect("window.history should resolve through host bindings");

    assert_eq!(host.window_history_length_calls, 2);
    assert_eq!(host.window_history_scroll_restoration_calls, 1);
    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(1))
            .map(String::as_str),
        Some("[object History]:1:1:null:auto")
    );
}

#[test]
fn runtime_updates_window_history_state_via_push_and_replace_state() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(1), "");
    host.window_history_length_result = 1;
    host.window_history_state_result = None;

    runtime
        .eval_program(
            "window.history.pushState('step-1', '', 'https://example.test/step-1'); window.history.replaceState('step-2', '', 'https://example.test/step-2'); document.getElementById('out').textContent = document.location + ':' + String(window.history.length) + ':' + String(window.history.state);",
            "inline-script",
            &mut host,
        )
        .expect("window.history.pushState and replaceState should resolve through host bindings");

    assert_eq!(
        host.window_history_push_state_calls,
        vec![(
            Some("step-1".to_string()),
            Some("https://example.test/step-1".to_string())
        )]
    );
    assert_eq!(
        host.window_history_replace_state_calls,
        vec![(
            Some("step-2".to_string()),
            Some("https://example.test/step-2".to_string())
        )]
    );
    assert_eq!(host.window_history_length_result, 2);
    assert_eq!(host.window_history_state_result.as_deref(), Some("step-2"));
    assert_eq!(host.document_location_result, "https://example.test/step-2");
    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(1))
            .map(String::as_str),
        Some("https://example.test/step-2:2:step-2")
    );
}

#[test]
fn runtime_rejects_window_history_assignment() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program("window.history.length = 2;", "inline-script", &mut host)
        .expect_err("window.history should be read-only");

    assert!(error.message().contains("history"));
    assert!(error.message().contains("length"));
}

#[test]
fn runtime_rejects_window_history_state_assignment() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program("window.history.state = 'step';", "inline-script", &mut host)
        .expect_err("window.history.state should be read-only");

    assert!(error.message().contains("history"));
    assert!(error.message().contains("state"));
}

#[test]
fn runtime_rejects_window_history_push_state_with_too_few_arguments() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program(
            "window.history.pushState('step');",
            "inline-script",
            &mut host,
        )
        .expect_err("window.history.pushState should reject too few arguments");

    assert!(
        error
            .message()
            .contains("history.pushState() expects 2 or 3 arguments")
    );
}

#[test]
fn runtime_rejects_window_history_replace_state_with_too_few_arguments() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program(
            "window.history.replaceState('step');",
            "inline-script",
            &mut host,
        )
        .expect_err("window.history.replaceState should reject too few arguments");

    assert!(
        error
            .message()
            .contains("history.replaceState() expects 2 or 3 arguments")
    );
}

#[test]
fn runtime_updates_window_history_scroll_restoration() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(1), "");
    host.window_history_scroll_restoration_result = "auto".to_string();

    runtime
        .eval_program(
            "window.history.scrollRestoration = 'manual'; document.getElementById('out').textContent = String(window.history.scrollRestoration);",
            "inline-script",
            &mut host,
        )
        .expect("window.history.scrollRestoration should be writable");

    assert_eq!(
        host.window_history_scroll_restoration_set_calls,
        vec!["manual".to_string()]
    );
    assert_eq!(host.window_history_scroll_restoration_result, "manual");
    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(1))
            .map(String::as_str),
        Some("manual")
    );
}

#[test]
fn runtime_rejects_window_history_scroll_restoration_assignment() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program(
            "window.history.scrollRestoration = 'sideways';",
            "inline-script",
            &mut host,
        )
        .expect_err("window.history.scrollRestoration should reject invalid values");

    assert!(error.message().contains("scroll restoration"));
}

#[test]
fn runtime_exposes_window_history_methods() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(1), "");
    host.window_history_length_result = 2;

    runtime
        .eval_program(
            "window.history.back(); window.history.forward(); window.history.go(-1); document.getElementById('out').textContent = String(window.history.length);",
            "inline-script",
            &mut host,
        )
        .expect("window.history methods should resolve through host bindings");

    assert_eq!(host.window_history_back_calls, 1);
    assert_eq!(host.window_history_forward_calls, 1);
    assert_eq!(host.window_history_go_calls, vec![-1]);
    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(1))
            .map(String::as_str),
        Some("2")
    );
}

#[test]
fn runtime_routes_window_dialogs_through_host_bindings() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(1), "");
    host.dialog_confirm_queue.push(true);
    host.dialog_prompt_queue.push(Some("Ada".to_string()));

    runtime
        .eval_program(
            "window.alert('Notice'); document.getElementById('out').textContent = String(window.confirm('Continue?')) + ':' + String(window.prompt('Name?', 'Default'));",
            "inline-script",
            &mut host,
        )
        .expect("window dialogs should resolve through host bindings");

    assert_eq!(host.dialog_alert_messages, vec!["Notice".to_string()]);
    assert_eq!(host.dialog_confirm_messages, vec!["Continue?".to_string()]);
    assert_eq!(host.dialog_prompt_messages, vec!["Name?".to_string()]);
    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(1))
            .map(String::as_str),
        Some("true:Ada")
    );
}

#[test]
fn runtime_rejects_window_confirm_without_queued_response() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program("window.confirm('Continue?');", "inline-script", &mut host)
        .expect_err("window.confirm should require a queued response");

    assert!(
        error
            .to_string()
            .contains("confirm() requires a queued response")
    );
}

#[test]
fn runtime_rejects_window_prompt_without_queued_response() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program("window.prompt('Name?');", "inline-script", &mut host)
        .expect_err("window.prompt should require a queued response");

    assert!(
        error
            .to_string()
            .contains("prompt() requires a queued response")
    );
}

#[test]
fn runtime_rejects_window_history_back_with_arguments() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program("window.history.back(1);", "inline-script", &mut host)
        .expect_err("window.history.back should reject arguments");

    assert!(
        error
            .message()
            .contains("history.back() expects no arguments")
    );
}

#[test]
fn runtime_resolves_window_match_media_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(1), "");
    host.seed_match_media_result("(prefers-color-scheme: dark)", true);

    runtime
        .eval_program(
            "const list = window.matchMedia('(prefers-color-scheme: dark)'); document.getElementById('out').textContent = String(list.matches) + ':' + list.media + ':' + String(window.matchMedia('(prefers-color-scheme: dark)'));",
            "inline-script",
            &mut host,
        )
        .expect("window.matchMedia should resolve through host bindings");

    assert_eq!(
        host.match_media_calls,
        vec![
            "(prefers-color-scheme: dark)".to_string(),
            "(prefers-color-scheme: dark)".to_string()
        ]
    );
    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(1))
            .map(String::as_str),
        Some("true:(prefers-color-scheme: dark):[object MediaQueryList]")
    );
}

#[test]
fn runtime_rejects_window_match_media_wrong_arity() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program("window.matchMedia();", "inline-script", &mut host)
        .expect_err("matchMedia should require exactly one argument");

    assert!(
        error
            .to_string()
            .contains("matchMedia() expects exactly one argument")
    );
}

#[test]
fn runtime_resolves_window_print_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(1), "");

    runtime
        .eval_program(
            "window.print(); document.getElementById('out').textContent = 'done';",
            "inline-script",
            &mut host,
        )
        .expect("window.print should resolve through host bindings");

    assert_eq!(host.window_print_calls, 1);
    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(1))
            .map(String::as_str),
        Some("done")
    );
}

#[test]
fn runtime_resolves_window_close_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(1), "");

    runtime
        .eval_program(
            "window.close(); document.getElementById('out').textContent = 'done';",
            "inline-script",
            &mut host,
        )
        .expect("window.close should resolve through host bindings");

    assert_eq!(host.window_close_calls, 1);
    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(1))
            .map(String::as_str),
        Some("done")
    );
}

#[test]
fn runtime_propagates_window_close_host_failures() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_window_close_error("window closed");

    let error = runtime
        .eval_program("window.close();", "inline-script", &mut host)
        .expect_err("window.close should propagate host failures");

    assert!(error.to_string().contains("window closed"));
    assert_eq!(host.window_close_calls, 1);
}

#[test]
fn runtime_rejects_window_close_wrong_arity() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program("window.close(1);", "inline-script", &mut host)
        .expect_err("window.close should reject arguments");

    assert!(error.to_string().contains("close() expects no arguments"));
    assert_eq!(host.window_close_calls, 0);
}

#[test]
fn runtime_propagates_window_print_host_failures() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_window_print_error("print blocked");

    let error = runtime
        .eval_program("window.print();", "inline-script", &mut host)
        .expect_err("window.print should propagate host failures");

    assert!(error.to_string().contains("print blocked"));
    assert_eq!(host.window_print_calls, 1);
}

#[test]
fn runtime_rejects_window_print_wrong_arity() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program("window.print(1);", "inline-script", &mut host)
        .expect_err("window.print should reject arguments");

    assert!(error.to_string().contains("print() expects no arguments"));
    assert_eq!(host.window_print_calls, 0);
}

#[test]
fn runtime_rejects_window_navigator_assignment() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program(
            "window.navigator.userAgent = 'x';",
            "inline-script",
            &mut host,
        )
        .expect_err("window.navigator should be read-only");

    assert!(
        error
            .to_string()
            .contains("cannot assign to `userAgent` on navigator value")
    );
}

#[test]
fn runtime_rejects_window_navigator_app_name_assignment() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program(
            "window.navigator.appName = 'x';",
            "inline-script",
            &mut host,
        )
        .expect_err("window.navigator.appName should be read-only");

    assert!(
        error
            .to_string()
            .contains("cannot assign to `appName` on navigator value")
    );
}

#[test]
fn runtime_rejects_window_navigator_app_code_name_assignment() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program(
            "window.navigator.appCodeName = 'x';",
            "inline-script",
            &mut host,
        )
        .expect_err("window.navigator.appCodeName should be read-only");

    assert!(
        error
            .to_string()
            .contains("cannot assign to `appCodeName` on navigator value")
    );
}

#[test]
fn runtime_rejects_window_navigator_app_version_assignment() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program(
            "window.navigator.appVersion = 'x';",
            "inline-script",
            &mut host,
        )
        .expect_err("window.navigator.appVersion should be read-only");

    assert!(
        error
            .to_string()
            .contains("cannot assign to `appVersion` on navigator value")
    );
}

#[test]
fn runtime_rejects_window_navigator_product_assignment() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program(
            "window.navigator.product = 'x';",
            "inline-script",
            &mut host,
        )
        .expect_err("window.navigator.product should be read-only");

    assert!(
        error
            .to_string()
            .contains("cannot assign to `product` on navigator value")
    );
}

#[test]
fn runtime_rejects_window_navigator_product_sub_assignment() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program(
            "window.navigator.productSub = 'x';",
            "inline-script",
            &mut host,
        )
        .expect_err("window.navigator.productSub should be read-only");

    assert!(
        error
            .to_string()
            .contains("cannot assign to `productSub` on navigator value")
    );
}

#[test]
fn runtime_rejects_window_navigator_on_line_assignment() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program(
            "window.navigator.onLine = false;",
            "inline-script",
            &mut host,
        )
        .expect_err("window.navigator.onLine should be read-only");

    assert!(
        error
            .to_string()
            .contains("cannot assign to `onLine` on navigator value")
    );
}

#[test]
fn runtime_rejects_window_navigator_webdriver_assignment() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program(
            "window.navigator.webdriver = true;",
            "inline-script",
            &mut host,
        )
        .expect_err("window.navigator.webdriver should be read-only");

    assert!(
        error
            .to_string()
            .contains("cannot assign to `webdriver` on navigator value")
    );
}

#[test]
fn runtime_rejects_window_navigator_vendor_assignment() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program("window.navigator.vendor = 'x';", "inline-script", &mut host)
        .expect_err("window.navigator.vendor should be read-only");

    assert!(
        error
            .to_string()
            .contains("cannot assign to `vendor` on navigator value")
    );
}

#[test]
fn runtime_rejects_window_navigator_vendor_sub_assignment() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program(
            "window.navigator.vendorSub = 'x';",
            "inline-script",
            &mut host,
        )
        .expect_err("window.navigator.vendorSub should be read-only");

    assert!(
        error
            .to_string()
            .contains("cannot assign to `vendorSub` on navigator value")
    );
}

#[test]
fn runtime_rejects_window_navigator_pdf_viewer_enabled_assignment() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program(
            "window.navigator.pdfViewerEnabled = true;",
            "inline-script",
            &mut host,
        )
        .expect_err("window.navigator.pdfViewerEnabled should be read-only");

    assert!(
        error
            .to_string()
            .contains("cannot assign to `pdfViewerEnabled` on navigator value")
    );
}

#[test]
fn runtime_rejects_window_navigator_do_not_track_assignment() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program(
            "window.navigator.doNotTrack = '1';",
            "inline-script",
            &mut host,
        )
        .expect_err("window.navigator.doNotTrack should be read-only");

    assert!(
        error
            .to_string()
            .contains("cannot assign to `doNotTrack` on navigator value")
    );
}

#[test]
fn runtime_rejects_window_navigator_languages_assignment() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program(
            "window.navigator.languages = null;",
            "inline-script",
            &mut host,
        )
        .expect_err("window.navigator.languages should be read-only");

    assert!(
        error
            .to_string()
            .contains("cannot assign to `languages` on navigator value")
    );
}

#[test]
fn runtime_rejects_window_navigator_languages_to_string_arguments() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program(
            "window.navigator.languages.toString(1);",
            "inline-script",
            &mut host,
        )
        .expect_err("window.navigator.languages.toString should reject arguments");

    assert!(
        error
            .to_string()
            .contains("navigator.languages.toString() expects no arguments")
    );
}

#[test]
fn runtime_rejects_window_navigator_user_language_assignment() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program(
            "window.navigator.userLanguage = 'fr-FR';",
            "inline-script",
            &mut host,
        )
        .expect_err("window.navigator.userLanguage should be read-only");

    assert!(
        error
            .to_string()
            .contains("cannot assign to `userLanguage` on navigator value")
    );
}

#[test]
fn runtime_rejects_window_navigator_system_language_assignment() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program(
            "window.navigator.systemLanguage = 'fr-FR';",
            "inline-script",
            &mut host,
        )
        .expect_err("window.navigator.systemLanguage should be read-only");

    assert!(
        error
            .to_string()
            .contains("cannot assign to `systemLanguage` on navigator value")
    );
}

#[test]
fn runtime_rejects_window_navigator_oscpu_assignment() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program("window.navigator.oscpu = 'x';", "inline-script", &mut host)
        .expect_err("window.navigator.oscpu should be read-only");

    assert!(
        error
            .to_string()
            .contains("cannot assign to `oscpu` on navigator value")
    );
}

#[test]
fn runtime_resolves_window_navigator_languages_contains_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_navigator("browser-tester-next", "unknown", "en-US", true, true);
    host.seed_element("out", ElementHandle::new(1), "");

    runtime
        .eval_program(
            "const languages = window.navigator.languages; document.getElementById('out').textContent = String(languages.contains('en-US')) + ':' + String(languages.contains('fr-FR'));",
            "inline-script",
            &mut host,
        )
        .expect("window.navigator.languages.contains should resolve through host bindings");

    assert_eq!(host.navigator_languages_calls, 1);
    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(1))
            .map(String::as_str),
        Some("true:false")
    );
}

#[test]
fn runtime_resolves_window_navigator_languages_iterator_helpers() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_navigator("browser-tester-next", "unknown", "en-US", true, true);
    host.seed_element("out", ElementHandle::new(1), "");

    runtime
        .eval_program(
            "const languages = window.navigator.languages; const keys = languages.keys(); const values = languages.values(); const entries = languages.entries(); const firstKey = keys.next(); const firstValue = values.next(); const firstEntry = entries.next(); const secondKey = keys.next(); const secondValue = values.next(); const secondEntry = entries.next(); document.getElementById('out').textContent = String(firstKey.value) + ':' + String(firstValue.value) + ':' + String(firstEntry.value.index) + ':' + firstEntry.value.value + ':' + String(secondKey.done) + ':' + String(secondValue.done) + ':' + String(secondEntry.done);",
            "inline-script",
            &mut host,
        )
        .expect("window.navigator.languages iterator helpers should resolve through host bindings");

    assert_eq!(host.navigator_languages_calls, 1);
    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(1))
            .map(String::as_str),
        Some("0:en-US:0:en-US:true:true:true")
    );
}

#[test]
fn runtime_rejects_window_navigator_mime_types_assignment() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program(
            "window.navigator.mimeTypes = null;",
            "inline-script",
            &mut host,
        )
        .expect_err("window.navigator.mimeTypes should be read-only");

    assert!(
        error
            .to_string()
            .contains("cannot assign to `mimeTypes` on navigator value")
    );
}

#[test]
fn runtime_rejects_window_navigator_plugins_assignment() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program(
            "window.navigator.plugins = null;",
            "inline-script",
            &mut host,
        )
        .expect_err("window.navigator.plugins should be read-only");

    assert!(
        error
            .to_string()
            .contains("cannot assign to `plugins` on navigator value")
    );
}

#[test]
fn runtime_resolves_window_navigator_mime_types_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = NoopHost::default();

    runtime
        .eval_program(
            "window.navigator.mimeTypes.length;",
            "inline-script",
            &mut host,
        )
        .expect("window.navigator.mimeTypes should resolve through host bindings");

    assert_eq!(host.navigator_mime_types_calls, 1);
}

#[test]
fn runtime_resolves_window_navigator_mime_types_named_item_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = NoopHost::default();

    runtime
        .eval_program(
            "window.navigator.mimeTypes.namedItem('missing');",
            "inline-script",
            &mut host,
        )
        .expect("window.navigator.mimeTypes.namedItem should resolve through host bindings");

    assert_eq!(host.navigator_mime_types_calls, 1);
}

#[test]
fn runtime_resolves_window_navigator_mime_types_iterator_helpers() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(1), "");

    runtime
        .eval_program(
            "const mimeTypes = window.navigator.mimeTypes; const keys = mimeTypes.keys(); const values = mimeTypes.values(); const entries = mimeTypes.entries(); const firstKey = keys.next(); const firstValue = values.next(); const firstEntry = entries.next(); document.getElementById('out').textContent = String(firstKey.done) + ':' + String(firstValue.done) + ':' + String(firstEntry.done);",
            "inline-script",
            &mut host,
        )
        .expect("window.navigator.mimeTypes iterator helpers should resolve through host bindings");

    assert_eq!(host.navigator_mime_types_calls, 1);
    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(1))
            .map(String::as_str),
        Some("true:true:true")
    );
}

#[test]
fn runtime_resolves_window_navigator_java_enabled_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(1), "");

    runtime
        .eval_program(
            "document.getElementById('out').textContent = String(window.navigator.javaEnabled());",
            "inline-script",
            &mut host,
        )
        .expect("window.navigator.javaEnabled should resolve through host bindings");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(1))
            .map(String::as_str),
        Some("false")
    );
}

#[test]
fn runtime_resolves_window_navigator_plugins_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("first-embed", ElementHandle::new(1), "");
    host.seed_element("second-embed", ElementHandle::new(2), "");
    host.seed_element("out", ElementHandle::new(3), "");
    let plugins_collection = HtmlCollectionTarget::ByTagName {
        scope: HtmlCollectionScope::Document,
        tag_name: "embed".to_string(),
    };
    host.seed_html_collection_tag_name_items(
        plugins_collection.clone(),
        vec![ElementHandle::new(1), ElementHandle::new(2)],
    );

    runtime
        .eval_program(
            "document.getElementById('out').textContent = String(window.navigator.plugins.length);",
            "inline-script",
            &mut host,
        )
        .expect("window.navigator.plugins should resolve through host bindings");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(3))
            .map(String::as_str),
        Some("2")
    );
    assert_eq!(
        host.html_collection_tag_name_items_calls,
        vec![plugins_collection]
    );
    assert_eq!(
        host.html_collection_tag_name_named_item_calls,
        Vec::<(HtmlCollectionTarget, String)>::new()
    );
}

#[test]
fn runtime_resolves_window_navigator_plugins_named_item_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("first-embed", ElementHandle::new(1), "");
    host.seed_element("second-embed", ElementHandle::new(2), "");
    host.seed_element("out", ElementHandle::new(3), "");
    let plugins_collection = HtmlCollectionTarget::ByTagName {
        scope: HtmlCollectionScope::Document,
        tag_name: "embed".to_string(),
    };
    host.seed_html_collection_tag_name_items(
        plugins_collection.clone(),
        vec![ElementHandle::new(1), ElementHandle::new(2)],
    );
    host.seed_html_collection_tag_name_named_item(
        plugins_collection.clone(),
        "first-embed",
        Some(ElementHandle::new(1)),
    );
    host.seed_html_collection_tag_name_named_item(plugins_collection.clone(), "missing", None);

    runtime
        .eval_program(
            "document.getElementById('out').textContent = String(window.navigator.plugins.namedItem('first-embed'));",
            "inline-script",
            &mut host,
        )
        .expect("window.navigator.plugins.namedItem should resolve through host bindings");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(3))
            .map(String::as_str),
        Some("[object Element]")
    );
    assert_eq!(
        host.html_collection_tag_name_items_calls,
        Vec::<HtmlCollectionTarget>::new()
    );
    assert_eq!(
        host.html_collection_tag_name_named_item_calls,
        vec![(plugins_collection, "first-embed".to_string())]
    );
}

#[test]
fn runtime_resolves_window_navigator_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(1), "");
    host.seed_navigator("browser-tester-next", "unknown", "en-US", true, true);

    runtime
            .eval_program(
            "document.getElementById('out').textContent = window.navigator.userAgent + ':' + window.navigator.appCodeName + ':' + window.navigator.appName + ':' + window.navigator.appVersion + ':' + window.navigator.product + ':' + window.navigator.productSub + ':' + window.navigator.vendor + ':' + window.navigator.vendorSub + ':' + String(window.navigator.pdfViewerEnabled) + ':' + window.navigator.doNotTrack + ':' + String(window.navigator.javaEnabled()) + ':' + String(window.navigator.plugins.length) + ':' + window.navigator.platform + ':' + window.navigator.language + ':' + window.navigator.oscpu + ':' + window.navigator.userLanguage + ':' + window.navigator.browserLanguage + ':' + window.navigator.systemLanguage + ':' + String(window.navigator.languages.length) + ':' + window.navigator.languages.item(0) + ':' + window.navigator.languages.toString() + ':' + String(window.navigator.cookieEnabled) + ':' + String(window.navigator.onLine) + ':' + String(window.navigator.webdriver) + ':' + String(window.navigator.hardwareConcurrency) + ':' + String(window.navigator.maxTouchPoints);",
            "inline-script",
            &mut host,
        )
        .expect("window.navigator should resolve through host bindings");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(1))
            .map(String::as_str),
        Some(
            "browser-tester-next:browser-tester-next:browser-tester-next:browser-tester-next:browser-tester-next:browser-tester-next:browser-tester-next:browser-tester-next:false:unspecified:false:0:unknown:en-US:unknown:en-US:en-US:en-US:1:en-US:[object DOMStringList]:true:true:false:8:0"
        )
    );
}

#[test]
fn runtime_resolves_window_scroll_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(1), "");

    runtime
        .eval_program(
            "window.scrollTo(10, 20); window.scrollBy(-5, 3); document.getElementById('out').textContent = String(window.scrollX) + ':' + String(window.scrollY) + ':' + String(window.pageXOffset) + ':' + String(window.pageYOffset);",
            "inline-script",
            &mut host,
        )
        .expect("window.scroll should resolve through host bindings");

    assert_eq!(
        host.window_scroll_calls,
        vec![
            ("scrollTo".to_string(), 10, 20),
            ("scrollBy".to_string(), -5, 3),
        ]
    );
    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(1))
            .map(String::as_str),
        Some("5:23:5:23")
    );
}

#[test]
fn runtime_propagates_window_scroll_host_failures() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_window_scroll_error("scroll blocked");

    let error = runtime
        .eval_program("window.scrollTo(1, 2);", "inline-script", &mut host)
        .expect_err("window.scrollTo should propagate host failures");

    assert!(error.to_string().contains("scroll blocked"));
    assert_eq!(
        host.window_scroll_calls,
        vec![("scrollTo".to_string(), 1, 2)]
    );
}

#[test]
fn runtime_rejects_window_scroll_wrong_arity() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program("window.scrollTo(1, 2, 3);", "inline-script", &mut host)
        .expect_err("window.scrollTo should reject too many arguments");

    assert!(
        error
            .to_string()
            .contains("scrollTo() expects at most two arguments")
    );
    assert!(host.window_scroll_calls.is_empty());
}

#[test]
fn runtime_resolves_window_open_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(1), "");

    runtime
        .eval_program(
            "window.open('https://example.test/popup', '_blank', 'noopener'); document.getElementById('out').textContent = 'done';",
            "inline-script",
            &mut host,
        )
        .expect("window.open should resolve through host bindings");

    assert_eq!(host.window_open_calls.len(), 1);
    assert_eq!(
        host.window_open_calls[0],
        (
            Some("https://example.test/popup".to_string()),
            Some("_blank".to_string()),
            Some("noopener".to_string())
        )
    );
    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(1))
            .map(String::as_str),
        Some("done")
    );
}

#[test]
fn runtime_propagates_window_open_host_failures() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_window_open_error("popup blocked");

    let error = runtime
        .eval_program(
            "window.open('https://example.test/popup');",
            "inline-script",
            &mut host,
        )
        .expect_err("window.open should propagate host failures");

    assert!(error.to_string().contains("popup blocked"));
    assert_eq!(host.window_open_calls.len(), 1);
}

#[test]
fn runtime_rejects_window_open_wrong_arity() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program("window.open(1, 2, 3, 4);", "inline-script", &mut host)
        .expect_err("window.open should reject too many arguments");

    assert!(
        error
            .to_string()
            .contains("open() expects at most three arguments")
    );
    assert_eq!(host.window_open_calls.len(), 0);
}

#[test]
fn runtime_resolves_document_dir_getter_and_setter_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("root", ElementHandle::new(1), "");
    host.seed_element("out", ElementHandle::new(2), "");
    host.seed_document_dir("ltr");

    runtime
        .eval_program(
            "const before = document.dir; document.dir = 'rtl'; const after = document.dir; document.getElementById('out').textContent = before + ':' + after;",
            "inline-script",
            &mut host,
        )
        .expect("document.dir should resolve through host bindings");

    assert_eq!(host.document_dir_calls, 2);
    assert_eq!(host.document_set_dir_calls, vec!["rtl".to_string()]);
    assert_eq!(host.document_dir_result, "rtl");
    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(2))
            .map(String::as_str),
        Some("ltr:rtl")
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
fn runtime_resolves_location_href_getter_and_setter() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(1), "");
    host.seed_document_location("https://example.test/start");

    runtime
        .eval_program(
            "const before = window.location.href; document.location.href = 'https://example.test/next'; const after = document.location.href; document.getElementById('out').textContent = before + ':' + after;",
            "inline-script",
            &mut host,
        )
        .expect("location.href should resolve through host bindings");

    assert_eq!(host.document_location_calls, 3);
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
fn runtime_resolves_location_hash_getter_and_setter() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(1), "");
    host.seed_document_location("https://example.test/start#old");

    runtime
        .eval_program(
            "const before = window.location.hash; document.location.hash = '#next'; const after = document.location.hash; document.getElementById('out').textContent = before + ':' + document.location + ':' + after;",
            "inline-script",
            &mut host,
        )
        .expect("location.hash should resolve through host bindings");

    assert_eq!(host.document_location_calls, 4);
    assert_eq!(
        host.document_set_location_calls,
        vec!["https://example.test/start#next".to_string()]
    );
    assert_eq!(
        host.document_location_result,
        "https://example.test/start#next"
    );
    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(1))
            .map(String::as_str),
        Some("#old:https://example.test/start#next:#next")
    );
}

#[test]
fn runtime_resolves_location_pathname_getter_and_setter() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(1), "");
    host.seed_document_location("https://example.test/start?x#old");

    runtime
        .eval_program(
            "const before = window.location.pathname; document.location.pathname = 'next'; const after = document.location.pathname; document.getElementById('out').textContent = before + ':' + document.location + ':' + after;",
            "inline-script",
            &mut host,
        )
        .expect("location.pathname should resolve through host bindings");

    assert_eq!(host.document_location_calls, 4);
    assert_eq!(
        host.document_set_location_calls,
        vec!["https://example.test/next?x#old".to_string()]
    );
    assert_eq!(
        host.document_location_result,
        "https://example.test/next?x#old"
    );
    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(1))
            .map(String::as_str),
        Some("/start:https://example.test/next?x#old:/next")
    );
}

#[test]
fn runtime_resolves_location_search_getter_and_setter() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(1), "");
    host.seed_document_location("https://example.test/start?x#old");

    runtime
        .eval_program(
            "const before = window.location.search; document.location.search = '?next'; const after = document.location.search; document.getElementById('out').textContent = before + ':' + document.location + ':' + after;",
            "inline-script",
            &mut host,
        )
        .expect("location.search should resolve through host bindings");

    assert_eq!(host.document_location_calls, 4);
    assert_eq!(
        host.document_set_location_calls,
        vec!["https://example.test/start?next#old".to_string()]
    );
    assert_eq!(
        host.document_location_result,
        "https://example.test/start?next#old"
    );
    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(1))
            .map(String::as_str),
        Some("?x:https://example.test/start?next#old:?next")
    );
}

#[test]
fn runtime_resolves_location_origin_getter() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(1), "");
    host.seed_document_location("https://example.test:8443/start?x#old");

    runtime
        .eval_program(
            "const before = window.location.origin; document.location.pathname = 'next'; const after = document.location.origin; document.getElementById('out').textContent = before + ':' + after + ':' + document.location;",
            "inline-script",
            &mut host,
        )
        .expect("location.origin should resolve through host bindings");

    assert_eq!(host.document_origin_calls, 2);
    assert_eq!(
        host.document_set_location_calls,
        vec!["https://example.test:8443/next?x#old".to_string()]
    );
    assert_eq!(
        host.document_location_result,
        "https://example.test:8443/next?x#old"
    );
    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(1))
            .map(String::as_str),
        Some(
            "https://example.test:8443:https://example.test:8443:https://example.test:8443/next?x#old"
        )
    );
}

#[test]
fn runtime_resolves_location_stringification_helpers() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(1), "");
    host.seed_document_location("https://example.test:8443/start?x#old");

    runtime
        .eval_program(
            "document.getElementById('out').textContent = document.location.toString() + ':' + window.location.valueOf();",
            "inline-script",
            &mut host,
        )
        .expect("location stringification should resolve through host bindings");

    assert_eq!(host.document_location_calls, 2);
    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(1))
            .map(String::as_str),
        Some("https://example.test:8443/start?x#old:https://example.test:8443/start?x#old")
    );
}

#[test]
fn runtime_resolves_location_protocol_host_hostname_and_port_getters_and_setters() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(1), "");
    host.seed_document_location("https://app.local:8443/start?x#old");

    runtime
        .eval_program(
            "const before = window.location.protocol + '|' + window.location.host + '|' + window.location.hostname + '|' + window.location.port; document.location.protocol = 'http:'; document.location.host = 'example.test:8080'; document.location.hostname = 'example.test'; document.location.port = '8080'; const after = window.location.protocol + '|' + window.location.host + '|' + window.location.hostname + '|' + window.location.port; document.getElementById('out').textContent = before + ':' + after + ':' + document.location;",
            "inline-script",
            &mut host,
        )
        .expect("location protocol/host/hostname/port should resolve through host bindings");

    assert_eq!(
        host.document_set_location_calls,
        vec![
            "http://app.local:8443/start?x#old".to_string(),
            "http://example.test:8080/start?x#old".to_string(),
            "http://example.test:8080/start?x#old".to_string(),
            "http://example.test:8080/start?x#old".to_string(),
        ]
    );
    assert_eq!(
        host.document_location_result,
        "http://example.test:8080/start?x#old"
    );
    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(1))
            .map(String::as_str),
        Some(
            "https:|app.local:8443|app.local|8443:http:|example.test:8080|example.test|8080:http://example.test:8080/start?x#old"
        )
    );
}

#[test]
fn runtime_resolves_location_username_and_password_getters_and_setters() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(1), "");
    host.seed_document_location("https://alice:secret@example.test:8443/start?x#old");

    runtime
        .eval_program(
            "const before = window.location.username + '|' + window.location.password; document.location.username = 'bob'; document.location.password = 'hunter2'; document.location.port = '9444'; const after = window.location.username + '|' + window.location.password + '|' + window.location.port; document.getElementById('out').textContent = before + ':' + after + ':' + document.location;",
            "inline-script",
            &mut host,
        )
        .expect("location username/password should resolve through host bindings");

    assert_eq!(
        host.document_set_location_calls,
        vec![
            "https://bob:secret@example.test:8443/start?x#old".to_string(),
            "https://bob:hunter2@example.test:8443/start?x#old".to_string(),
            "https://bob:hunter2@example.test:9444/start?x#old".to_string(),
        ]
    );
    assert_eq!(
        host.document_location_result,
        "https://bob:hunter2@example.test:9444/start?x#old"
    );
    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(1))
            .map(String::as_str),
        Some("alice|secret:bob|hunter2|9444:https://bob:hunter2@example.test:9444/start?x#old")
    );
}

#[test]
fn runtime_rejects_location_port_with_non_numeric_value() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_document_location("https://example.test:8443/start?x#old");

    let error = runtime
        .eval_program(
            "document.location.port = 'abc';",
            "inline-script",
            &mut host,
        )
        .expect_err("location.port should reject non-numeric values");

    assert!(
        error
            .to_string()
            .contains("unsupported location.port value: abc")
    );
    assert!(host.document_set_location_calls.is_empty());
    assert_eq!(
        host.document_location_result,
        "https://example.test:8443/start?x#old"
    );
}

#[test]
fn runtime_resolves_location_assign_replace_and_reload_calls() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(1), "");
    host.seed_document_location("https://example.test/start");

    runtime
        .eval_program(
            "const before = document.location; window.location.assign('https://example.test/assign'); document.location.replace('https://example.test/replace'); window.location.reload(); document.getElementById('out').textContent = before + ':' + document.location;",
            "inline-script",
            &mut host,
        )
        .expect("location methods should resolve through host bindings");

    assert_eq!(
        host.document_location_assign_calls,
        vec!["https://example.test/assign".to_string()]
    );
    assert_eq!(
        host.document_location_replace_calls,
        vec!["https://example.test/replace".to_string()]
    );
    assert_eq!(
        host.document_location_reload_calls,
        vec!["https://example.test/replace".to_string()]
    );
    assert_eq!(
        host.document_location_result,
        "https://example.test/replace"
    );
    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(1))
            .map(String::as_str),
        Some("https://example.test/start:https://example.test/replace")
    );
}

#[test]
fn runtime_rejects_location_assign_without_url() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program("window.location.assign();", "inline-script", &mut host)
        .expect_err("location.assign should require a URL");

    assert!(
        error
            .to_string()
            .contains("location.assign() expects exactly one argument")
    );
    assert!(host.document_location_assign_calls.is_empty());
}

#[test]
fn runtime_resolves_document_cookie_getter_and_setter() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(1), "");

    runtime
        .eval_program(
            "document.cookie = 'theme=dark'; document.cookie = 'theme=light'; document.getElementById('out').textContent = document.cookie;",
            "inline-script",
            &mut host,
        )
        .expect("document.cookie should resolve through host bindings");

    assert_eq!(
        host.document_set_cookie_calls,
        vec!["theme=dark".to_string(), "theme=light".to_string()]
    );
    assert_eq!(host.document_cookie_calls, 1);
    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(1))
            .map(String::as_str),
        Some("theme=light")
    );
}

#[test]
fn runtime_rejects_malformed_document_cookie_assignment() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program("document.cookie = 'badcookie';", "inline-script", &mut host)
        .expect_err("document.cookie should reject malformed assignments");

    assert!(
        error
            .to_string()
            .contains("document.cookie requires `name=value`")
    );
    assert_eq!(
        host.document_set_cookie_calls,
        vec!["badcookie".to_string()]
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
            "const root = document.getElementById('root'); const child = document.getElementById('child'); document.getElementById('root').textContent = document.domain + ':' + document.origin + ':' + window.origin + ':' + root.origin + ':' + child.origin;",
            "inline-script",
            &mut host,
        )
        .expect("document.domain and origin should resolve through host bindings");

    assert_eq!(host.document_domain_calls, 1);
    assert_eq!(host.document_origin_calls, 2);
    assert_eq!(
        host.element_origin_calls,
        vec![ElementHandle::new(1), ElementHandle::new(2)]
    );
    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(1))
            .map(String::as_str),
        Some(
            "example.test:https://example.test:8443:https://example.test:8443:https://example.test:8443:https://example.test:8443"
        )
    );
}

#[test]
fn runtime_rejects_document_domain_assignment() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program(
            "document.domain = 'example.test';",
            "inline-script",
            &mut host,
        )
        .expect_err("document.domain should be read-only");

    assert!(error.message().contains("unsupported assignment target"));
    assert!(error.message().contains("domain"));
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
fn runtime_dispatches_insert_adjacent_element_and_text() {
    struct AdjacentHost {
        create_element_calls: Vec<String>,
        create_text_node_calls: Vec<String>,
        node_parent_results: BTreeMap<NodeHandle, Option<NodeHandle>>,
        element_tag_name_results: BTreeMap<ElementHandle, String>,
        element_before_calls: Vec<(ElementHandle, Vec<NodeHandle>)>,
        element_after_calls: Vec<(ElementHandle, Vec<NodeHandle>)>,
        element_prepend_calls: Vec<(ElementHandle, Vec<NodeHandle>)>,
        element_append_calls: Vec<(ElementHandle, Vec<NodeHandle>)>,
    }

    impl HostBindings for AdjacentHost {
        fn document_get_element_by_id(
            &mut self,
            id: &str,
        ) -> bt_script::Result<Option<ElementHandle>> {
            Ok(match id {
                "target" => Some(ElementHandle::new(11)),
                _ => None,
            })
        }

        fn document_create_element(&mut self, tag_name: &str) -> bt_script::Result<ElementHandle> {
            self.create_element_calls.push(tag_name.to_string());
            Ok(match self.create_element_calls.len() {
                1 => ElementHandle::new(21),
                2 => ElementHandle::new(22),
                _ => ElementHandle::new(99),
            })
        }

        fn document_create_text_node(&mut self, text: &str) -> bt_script::Result<NodeHandle> {
            self.create_text_node_calls.push(text.to_string());
            Ok(match self.create_text_node_calls.len() {
                1 => NodeHandle::new(31),
                2 => NodeHandle::new(32),
                _ => NodeHandle::new(99),
            })
        }

        fn node_parent(&mut self, node: NodeHandle) -> bt_script::Result<Option<NodeHandle>> {
            Ok(self.node_parent_results.get(&node).copied().unwrap_or(None))
        }

        fn element_tag_name(&mut self, element: ElementHandle) -> bt_script::Result<String> {
            Ok(self
                .element_tag_name_results
                .get(&element)
                .cloned()
                .unwrap_or_else(|| "section".to_string()))
        }

        fn element_before(
            &mut self,
            element: ElementHandle,
            children: Vec<NodeHandle>,
        ) -> bt_script::Result<()> {
            self.element_before_calls.push((element, children));
            Ok(())
        }

        fn element_after(
            &mut self,
            element: ElementHandle,
            children: Vec<NodeHandle>,
        ) -> bt_script::Result<()> {
            self.element_after_calls.push((element, children));
            Ok(())
        }

        fn element_prepend(
            &mut self,
            element: ElementHandle,
            children: Vec<NodeHandle>,
        ) -> bt_script::Result<()> {
            self.element_prepend_calls.push((element, children));
            Ok(())
        }

        fn element_append(
            &mut self,
            element: ElementHandle,
            children: Vec<NodeHandle>,
        ) -> bt_script::Result<()> {
            self.element_append_calls.push((element, children));
            Ok(())
        }
    }

    let mut runtime = ScriptRuntime::new();
    let mut host = AdjacentHost {
        create_element_calls: Vec::new(),
        create_text_node_calls: Vec::new(),
        node_parent_results: BTreeMap::from([(NodeHandle::new(11), Some(NodeHandle::new(10)))]),
        element_tag_name_results: BTreeMap::from([(ElementHandle::new(11), "section".to_string())]),
        element_before_calls: Vec::new(),
        element_after_calls: Vec::new(),
        element_prepend_calls: Vec::new(),
        element_append_calls: Vec::new(),
    };

    runtime
        .eval_program(
            "const target = document.getElementById('target'); target.insertAdjacentElement('beforebegin', document.createElement('aside')); target.insertAdjacentElement('afterend', document.createElement('article')); target.insertAdjacentText('afterbegin', 'Hello'); target.insertAdjacentText('beforeend', 'Tail');",
            "inline-script",
            &mut host,
        )
        .expect("insertAdjacentElement and insertAdjacentText should resolve through host bindings");

    assert_eq!(host.create_element_calls, vec!["aside", "article"]);
    assert_eq!(host.create_text_node_calls, vec!["Hello", "Tail"]);
    assert_eq!(
        host.element_before_calls,
        vec![(ElementHandle::new(11), vec![NodeHandle::new(21)])]
    );
    assert_eq!(
        host.element_after_calls,
        vec![(ElementHandle::new(11), vec![NodeHandle::new(22)])]
    );
    assert_eq!(
        host.element_prepend_calls,
        vec![(ElementHandle::new(11), vec![NodeHandle::new(31)])]
    );
    assert_eq!(
        host.element_append_calls,
        vec![(ElementHandle::new(11), vec![NodeHandle::new(32)])]
    );
}

#[test]
fn runtime_rejects_insert_adjacent_element_invalid_position() {
    struct InvalidPositionHost;

    impl HostBindings for InvalidPositionHost {
        fn document_get_element_by_id(
            &mut self,
            id: &str,
        ) -> bt_script::Result<Option<ElementHandle>> {
            Ok(match id {
                "target" => Some(ElementHandle::new(11)),
                "child" => Some(ElementHandle::new(12)),
                _ => None,
            })
        }
    }

    let mut runtime = ScriptRuntime::new();
    let mut host = InvalidPositionHost;

    let error = runtime
        .eval_program(
            "document.getElementById('target').insertAdjacentElement('middle', document.getElementById('child'));",
            "inline-script",
            &mut host,
        )
        .expect_err("insertAdjacentElement should validate positions");

    assert!(
        error
            .to_string()
            .contains("unsupported insertAdjacentElement position")
    );
}

#[test]
fn runtime_rejects_insert_adjacent_text_on_void_element() {
    struct VoidElementHost;

    impl HostBindings for VoidElementHost {
        fn document_get_element_by_id(
            &mut self,
            id: &str,
        ) -> bt_script::Result<Option<ElementHandle>> {
            Ok(match id {
                "image" => Some(ElementHandle::new(11)),
                _ => None,
            })
        }

        fn element_tag_name(&mut self, element: ElementHandle) -> bt_script::Result<String> {
            Ok(match element.raw() {
                11 => "img".to_string(),
                _ => "div".to_string(),
            })
        }
    }

    let mut runtime = ScriptRuntime::new();
    let mut host = VoidElementHost;

    let error = runtime
        .eval_program(
            "document.getElementById('image').insertAdjacentText('beforeend', 'Bad');",
            "inline-script",
            &mut host,
        )
        .expect_err("insertAdjacentText should reject void elements");

    assert!(
        error
            .to_string()
            .contains("insertAdjacentText is not supported on void elements")
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
fn runtime_resolves_form_elements_radio_node_list_entries_access() {
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
            "const elements = document.getElementById('signup').elements; const named = elements.namedItem('mode'); const entries = named.entries(); const first = entries.next(); const second = entries.next(); const third = entries.next(); document.getElementById('out').textContent = String(named.length) + ':' + String(first.value.index) + ':' + first.value.value + ':' + String(second.value.index) + ':' + second.value.value + ':' + String(third.done);",
            "inline-script",
            &mut host,
        )
        .expect("radio node list entries should resolve");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(4))
            .map(String::as_str),
        Some("2:0:[object Element]:1:[object Element]:true")
    );
}

#[test]
fn runtime_sets_form_elements_radio_node_list_value() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("signup", ElementHandle::new(1), "Signup");
    host.seed_element("radio-a", ElementHandle::new(2), "");
    host.seed_element("radio-b", ElementHandle::new(3), "");
    host.seed_element("radio-c", ElementHandle::new(4), "");
    host.seed_element("out", ElementHandle::new(5), "");
    host.seed_element_tag_name(ElementHandle::new(2), "input");
    host.seed_element_tag_name(ElementHandle::new(3), "input");
    host.seed_element_tag_name(ElementHandle::new(4), "input");
    host.seed_attribute(ElementHandle::new(2), "type", "radio");
    host.seed_attribute(ElementHandle::new(3), "type", "radio");
    host.seed_attribute(ElementHandle::new(4), "type", "radio");
    host.seed_value(ElementHandle::new(2), "a");
    host.seed_value(ElementHandle::new(3), "b");
    host.seed_value(ElementHandle::new(4), "c");
    host.seed_checked(ElementHandle::new(2), true);
    host.seed_checked(ElementHandle::new(3), false);
    host.seed_checked(ElementHandle::new(4), false);

    host.seed_html_collection_form_elements_items(
        ElementHandle::new(1),
        vec![
            ElementHandle::new(2),
            ElementHandle::new(3),
            ElementHandle::new(4),
        ],
    );
    host.seed_html_collection_form_elements_named_items(
        ElementHandle::new(1),
        "mode",
        vec![
            ElementHandle::new(2),
            ElementHandle::new(3),
            ElementHandle::new(4),
        ],
    );

    runtime
        .eval_program(
            "const named = document.getElementById('signup').elements.namedItem('mode'); named.value = 'b'; document.getElementById('out').textContent = named.value + ':' + String(named.item(0).checked) + ':' + String(named.item(1).checked) + ':' + String(named.item(2).checked);",
            "inline-script",
            &mut host,
        )
        .expect("radio node list value should be assignable");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(5))
            .map(String::as_str),
        Some("b:false:true:false")
    );
    assert_eq!(host.checked.get(&ElementHandle::new(2)), Some(&false));
    assert_eq!(host.checked.get(&ElementHandle::new(3)), Some(&true));
    assert_eq!(host.checked.get(&ElementHandle::new(4)), Some(&false));
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
fn runtime_resolves_window_frames_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("first-frame", ElementHandle::new(1), "");
    host.seed_element("second-frame", ElementHandle::new(2), "");
    host.seed_element("out", ElementHandle::new(3), "");
    host.seed_attribute(ElementHandle::new(1), "id", "first");
    host.seed_attribute(ElementHandle::new(2), "id", "second");
    host.seed_window_frames_items(vec![ElementHandle::new(1), ElementHandle::new(2)]);
    host.seed_window_frames_named_item("first", Some(ElementHandle::new(1)));
    host.seed_window_frames_named_item("missing", None);

    runtime
        .eval_program(
            "const frames = window.frames; document.getElementById('out').textContent = String(frames.length) + ':' + frames.item(0).getAttribute('id') + ':' + frames.namedItem('first').getAttribute('id') + ':' + String(frames.namedItem('missing'));",
            "inline-script",
            &mut host,
        )
        .expect("window.frames should resolve");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(3))
            .map(String::as_str),
        Some("2:first:first:null")
    );
    assert_eq!(host.window_frames_items_calls, 2);
    assert_eq!(
        host.window_frames_named_item_calls,
        vec!["first".to_string(), "missing".to_string()]
    );
}

#[test]
fn runtime_resolves_window_frame_element_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(3), "");

    runtime
        .eval_program(
            "document.getElementById('out').textContent = String(window.frameElement);",
            "inline-script",
            &mut host,
        )
        .expect("window.frameElement should resolve");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(3))
            .map(String::as_str),
        Some("null")
    );
}

#[test]
fn runtime_rejects_window_frame_element_assignment() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program("window.frameElement = 2;", "inline-script", &mut host)
        .expect_err("window.frameElement should be read-only");

    assert!(error.to_string().contains("unsupported assignment target"));
}

#[test]
fn runtime_resolves_window_opener_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(3), "");

    runtime
        .eval_program(
            "document.getElementById('out').textContent = String(window.opener);",
            "inline-script",
            &mut host,
        )
        .expect("window.opener should resolve");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(3))
            .map(String::as_str),
        Some("null")
    );
}

#[test]
fn runtime_rejects_window_opener_assignment() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program("window.opener = 2;", "inline-script", &mut host)
        .expect_err("window.opener should be read-only");

    assert!(error.to_string().contains("unsupported assignment target"));
}

#[test]
fn runtime_resolves_form_and_select_length_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("signup", ElementHandle::new(1), "");
    host.seed_element_tag_name(ElementHandle::new(1), "form");
    host.seed_html_collection_form_elements_items(
        ElementHandle::new(1),
        vec![ElementHandle::new(4), ElementHandle::new(5)],
    );
    host.seed_element("mode", ElementHandle::new(2), "");
    host.seed_element_tag_name(ElementHandle::new(2), "select");
    host.seed_html_collection_select_options_items(
        ElementHandle::new(2),
        vec![ElementHandle::new(6), ElementHandle::new(7)],
    );
    host.seed_element("out", ElementHandle::new(3), "");

    runtime
        .eval_program(
            "document.getElementById('out').textContent = String(document.getElementById('signup').length) + ':' + String(document.getElementById('mode').length);",
            "inline-script",
            &mut host,
        )
        .expect("form.length and select.length should resolve");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(3))
            .map(String::as_str),
        Some("2:2")
    );
}

#[test]
fn runtime_rejects_form_length_assignment() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("signup", ElementHandle::new(1), "");

    let error = runtime
        .eval_program(
            "document.getElementById('signup').length = 2;",
            "inline-script",
            &mut host,
        )
        .expect_err("form.length should be read-only");

    assert!(error.to_string().contains("unsupported assignment target"));
    assert!(error.to_string().contains("element"));
    assert!(error.to_string().contains("length"));
}

#[test]
fn runtime_resolves_window_length_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(3), "");
    host.seed_window_frames_items(vec![ElementHandle::new(1), ElementHandle::new(2)]);

    runtime
        .eval_program(
            "document.getElementById('out').textContent = String(window.length);",
            "inline-script",
            &mut host,
        )
        .expect("window.length should resolve");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(3))
            .map(String::as_str),
        Some("2")
    );
    assert_eq!(host.window_frames_items_calls, 1);
}

#[test]
fn runtime_rejects_window_length_assignment() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program("window.length = 2;", "inline-script", &mut host)
        .expect_err("window.length should be read-only");

    assert!(error.to_string().contains("unsupported assignment target"));
}

#[test]
fn runtime_rejects_window_frames_length_assignment() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program("window.frames.length = 2;", "inline-script", &mut host)
        .expect_err("window.frames.length should be read-only");

    assert!(
        error
            .to_string()
            .contains("cannot assign to `length` on html collection value")
    );
}

#[test]
fn runtime_resolves_window_children_alias_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("root", ElementHandle::new(1), "First");
    host.seed_element("out", ElementHandle::new(2), "Second");
    host.seed_document_children_items(vec![ElementHandle::new(1), ElementHandle::new(2)]);
    host.seed_document_children_named_item("root", Some(ElementHandle::new(1)));
    host.seed_document_children_named_item("missing", None);

    runtime
        .eval_program(
            "const children = document.defaultView.children; document.getElementById('out').textContent = String(children.length) + ':' + children.item(0).textContent + ':' + children.item(1).textContent + ':' + String(children.namedItem('root')) + ':' + String(children.namedItem('missing'));",
            "inline-script",
            &mut host,
        )
        .expect("window.children should resolve as an alias of document.children");

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
    host.seed_node_child_nodes_items(
        HtmlCollectionScope::Node(NodeHandle::new(11)),
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
            "const docNodes = document.childNodes; const rootNode = docNodes.item(1); const rootNodes = rootNode.childNodes; const docFirst = docNodes.item(0); const docSecond = docNodes.item(1); const rootValues = rootNodes.values(); const firstRoot = rootValues.next(); const secondRoot = rootValues.next(); const thirdRoot = rootValues.next(); document.getElementById('out').textContent = String(docNodes.length) + ':' + docFirst.nodeName + ':' + String(docFirst.nodeType) + ':' + String(docFirst) + ':' + docSecond.nodeName + ':' + String(docSecond.nodeType) + ':' + rootNode.nodeName + ':' + firstRoot.value.nodeName + ':' + String(firstRoot.value.nodeType) + ':' + firstRoot.value.textContent + ':' + secondRoot.value.nodeName + ':' + String(secondRoot.value.nodeType) + ':' + secondRoot.value.textContent + ':' + thirdRoot.value.nodeName + ':' + String(thirdRoot.value.nodeType) + ':' + thirdRoot.value.textContent;",
            "inline-script",
            &mut host,
        )
        .expect("childNodes should resolve");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(2))
            .map(String::as_str),
        Some("2:#comment:8:[object Node]:main:1:main:#text:3:Hello:span:1:Inner:#comment:8:")
    );
    assert_eq!(
        host.node_child_nodes_items_calls,
        vec![
            HtmlCollectionScope::Document,
            HtmlCollectionScope::Document,
            HtmlCollectionScope::Document,
            HtmlCollectionScope::Node(NodeHandle::new(11)),
            HtmlCollectionScope::Document
        ]
    );
    assert_eq!(
        host.node_name_calls,
        vec![
            NodeHandle::new(10),
            NodeHandle::new(11),
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
fn runtime_resolves_template_content_get_element_by_id_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("tpl", ElementHandle::new(3), "");
    host.seed_element("out", ElementHandle::new(2), "");
    host.seed_element_tag_name(ElementHandle::new(3), "template");
    host.seed_element("foo,bar", ElementHandle::new(40), "First");
    host.seed_element_query_selector(
        ElementHandle::new(3),
        r"#foo\2c bar",
        Some(ElementHandle::new(40)),
    );

    runtime
        .eval_program(
            "const tpl = document.getElementById('tpl'); const content = tpl.content; const hit = content.getElementById('foo,bar'); document.getElementById('out').textContent = String(hit) + ':' + hit.textContent;",
            "inline-script",
            &mut host,
        )
        .expect("template content getElementById should resolve");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(2))
            .map(String::as_str),
        Some("[object Element]:First")
    );
    assert_eq!(
        host.element_query_selector_calls,
        vec![(ElementHandle::new(3), r"#foo\2c bar".to_string())]
    );
}

#[test]
fn runtime_rejects_template_content_get_element_by_id_wrong_arity() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("tpl", ElementHandle::new(3), "");
    host.seed_element_tag_name(ElementHandle::new(3), "template");

    let error = runtime
        .eval_program(
            "const tpl = document.getElementById('tpl'); tpl.content.getElementById();",
            "inline-script",
            &mut host,
        )
        .expect_err("template content getElementById should reject missing arguments");

    assert!(
        error
            .message()
            .contains("getElementById() expects exactly one argument")
    );
}

#[test]
fn runtime_resolves_template_content_query_selector_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("tpl", ElementHandle::new(3), "");
    host.seed_element("out", ElementHandle::new(2), "");
    host.seed_element_tag_name(ElementHandle::new(3), "template");
    host.seed_element("first", ElementHandle::new(40), "First");
    host.seed_element("second", ElementHandle::new(41), "Second");
    host.seed_element_query_selector(
        ElementHandle::new(3),
        ".primary",
        Some(ElementHandle::new(40)),
    );
    host.seed_element_query_selector_all(
        ElementHandle::new(3),
        ".primary",
        vec![ElementHandle::new(40), ElementHandle::new(41)],
    );

    runtime
        .eval_program(
            "const tpl = document.getElementById('tpl'); const content = tpl.content; const first = content.querySelector('.primary'); const all = content.querySelectorAll('.primary'); document.getElementById('out').textContent = String(content) + ':' + first.textContent + ':' + String(all.length) + ':' + all.item(0).textContent + ':' + all.item(1).textContent + ':' + String(all.item(2));",
            "inline-script",
            &mut host,
        )
        .expect("template content query selectors should resolve");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(2))
            .map(String::as_str),
        Some("[object DocumentFragment]:First:2:First:Second:null")
    );
    assert_eq!(
        host.element_query_selector_calls,
        vec![(ElementHandle::new(3), ".primary".to_string())]
    );
    assert_eq!(
        host.element_query_selector_all_calls,
        vec![(ElementHandle::new(3), ".primary".to_string())]
    );
}

#[test]
fn runtime_rejects_template_content_query_selector_all_wrong_arity() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("tpl", ElementHandle::new(3), "");
    host.seed_element_tag_name(ElementHandle::new(3), "template");

    let error = runtime
        .eval_program(
            "const tpl = document.getElementById('tpl'); tpl.content.querySelectorAll();",
            "inline-script",
            &mut host,
        )
        .expect_err("template content querySelectorAll should reject missing arguments");

    assert!(
        error
            .message()
            .contains("querySelectorAll() expects exactly one argument")
    );
}

#[test]
fn runtime_resolves_template_content_clone_node_access() {
    struct CloneHost {
        clone_called: bool,
    }

    impl HostBindings for CloneHost {
        fn document_get_element_by_id(
            &mut self,
            id: &str,
        ) -> bt_script::Result<Option<ElementHandle>> {
            Ok(match id {
                "tpl" => Some(ElementHandle::new(3)),
                _ => None,
            })
        }

        fn element_tag_name(&mut self, element: ElementHandle) -> bt_script::Result<String> {
            Ok(match element.raw() {
                3 | 4 => "template".to_string(),
                _ => "div".to_string(),
            })
        }

        fn node_clone(&mut self, node: NodeHandle, _deep: bool) -> bt_script::Result<NodeHandle> {
            self.clone_called = true;
            Ok(NodeHandle::new(node.raw() + 1))
        }

        fn node_type(&mut self, node: NodeHandle) -> bt_script::Result<u8> {
            Ok(match node.raw() {
                3 | 4 => 1,
                _ => 0,
            })
        }
    }

    let mut runtime = ScriptRuntime::new();
    let mut host = CloneHost {
        clone_called: false,
    };

    runtime
        .eval_program(
            "const tpl = document.getElementById('tpl'); tpl.content.cloneNode(true);",
            "inline-script",
            &mut host,
        )
        .expect("template content cloneNode should resolve");

    assert!(host.clone_called);
}

#[test]
fn runtime_rejects_template_content_clone_node_wrong_arity() {
    struct CloneHost;

    impl HostBindings for CloneHost {
        fn document_get_element_by_id(
            &mut self,
            id: &str,
        ) -> bt_script::Result<Option<ElementHandle>> {
            Ok(match id {
                "tpl" => Some(ElementHandle::new(3)),
                _ => None,
            })
        }

        fn element_tag_name(&mut self, element: ElementHandle) -> bt_script::Result<String> {
            Ok(match element.raw() {
                3 => "template".to_string(),
                _ => "div".to_string(),
            })
        }
    }

    let mut runtime = ScriptRuntime::new();
    let mut host = CloneHost;

    let error = runtime
        .eval_program(
            "const tpl = document.getElementById('tpl'); tpl.content.cloneNode(true, false);",
            "inline-script",
            &mut host,
        )
        .expect_err("template content cloneNode should validate arity");

    assert!(
        error
            .message()
            .contains("cloneNode() expects at most one argument")
    );
}

#[test]
fn runtime_resolves_template_content_append_child_access() {
    struct MutationHost {
        clone_called: bool,
        append_child_calls: Vec<(ElementHandle, NodeHandle)>,
    }

    impl HostBindings for MutationHost {
        fn document_get_element_by_id(
            &mut self,
            id: &str,
        ) -> bt_script::Result<Option<ElementHandle>> {
            Ok(match id {
                "tpl" => Some(ElementHandle::new(3)),
                _ => None,
            })
        }

        fn element_tag_name(&mut self, element: ElementHandle) -> bt_script::Result<String> {
            Ok(match element.raw() {
                3 | 4 => "template".to_string(),
                _ => "div".to_string(),
            })
        }

        fn node_clone(&mut self, node: NodeHandle, _deep: bool) -> bt_script::Result<NodeHandle> {
            self.clone_called = true;
            Ok(NodeHandle::new(node.raw() + 1))
        }

        fn node_child_nodes_items(
            &mut self,
            scope: HtmlCollectionScope,
        ) -> bt_script::Result<Vec<NodeHandle>> {
            Ok(match scope {
                HtmlCollectionScope::Element(element) if element.raw() == 4 => {
                    vec![NodeHandle::new(10)]
                }
                _ => Vec::new(),
            })
        }

        fn node_type(&mut self, node: NodeHandle) -> bt_script::Result<u8> {
            Ok(match node.raw() {
                3 | 4 | 10 => 1,
                _ => 0,
            })
        }

        fn element_append_child(
            &mut self,
            parent: ElementHandle,
            child: NodeHandle,
        ) -> bt_script::Result<()> {
            self.append_child_calls.push((parent, child));
            Ok(())
        }
    }

    let mut runtime = ScriptRuntime::new();
    let mut host = MutationHost {
        clone_called: false,
        append_child_calls: Vec::new(),
    };

    runtime
        .eval_program(
            "const tpl = document.getElementById('tpl'); const clone = tpl.content.cloneNode(true); const child = clone.childNodes.item(0); tpl.content.appendChild(child);",
            "inline-script",
            &mut host,
        )
        .expect("template content appendChild should resolve");

    assert!(host.clone_called);
    assert_eq!(
        host.append_child_calls,
        vec![(ElementHandle::new(3), NodeHandle::new(10))]
    );
}

#[test]
fn runtime_rejects_template_content_append_child_wrong_arity() {
    struct MutationHost;

    impl HostBindings for MutationHost {
        fn document_get_element_by_id(
            &mut self,
            id: &str,
        ) -> bt_script::Result<Option<ElementHandle>> {
            Ok(match id {
                "tpl" => Some(ElementHandle::new(3)),
                _ => None,
            })
        }

        fn element_tag_name(&mut self, element: ElementHandle) -> bt_script::Result<String> {
            Ok(match element.raw() {
                3 => "template".to_string(),
                _ => "div".to_string(),
            })
        }
    }

    let mut runtime = ScriptRuntime::new();
    let mut host = MutationHost;

    let error = runtime
        .eval_program(
            "const tpl = document.getElementById('tpl'); tpl.content.appendChild();",
            "inline-script",
            &mut host,
        )
        .expect_err("template content appendChild should validate arity");

    assert!(
        error
            .message()
            .contains("appendChild() expects exactly one argument")
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
fn runtime_resolves_template_content_text_content_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("tpl", ElementHandle::new(3), "");
    host.seed_element("out", ElementHandle::new(2), "");
    host.seed_element_tag_name(ElementHandle::new(3), "template");
    host.text_content
        .insert(ElementHandle::new(3), "Inner".to_string());

    runtime
        .eval_program(
            "const tpl = document.getElementById('tpl'); const content = tpl.content; const before = content.textContent; content.textContent = 'Updated'; document.getElementById('out').textContent = before + ':' + content.textContent;",
            "inline-script",
            &mut host,
        )
        .expect("template content textContent should resolve");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(2))
            .map(String::as_str),
        Some("Inner:Updated")
    );
    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(3))
            .map(String::as_str),
        Some("Updated")
    );
}

#[test]
fn runtime_resolves_template_content_fragment_reflection_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("tpl", ElementHandle::new(3), "");
    host.seed_element("out", ElementHandle::new(2), "");
    host.seed_element_tag_name(ElementHandle::new(3), "template");

    runtime
        .eval_program(
            "const content = document.getElementById('tpl').content; document.getElementById('out').textContent = String(content.nodeType) + ':' + content.nodeName + ':' + String(content.parentNode) + ':' + String(content.ownerDocument);",
            "inline-script",
            &mut host,
        )
        .expect("template content fragment reflection should resolve");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(2))
            .map(String::as_str),
        Some("11:#document-fragment:null:[object Document]")
    );
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
fn runtime_resolves_document_style_sheets_entries_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(3), "");
    host.seed_document_style_sheets_items(vec![ElementHandle::new(1), ElementHandle::new(2)]);

    runtime
        .eval_program(
            "const sheets = document.styleSheets; const keys = sheets.keys(); const values = sheets.values(); const entries = sheets.entries(); const key = keys.next(); const value = values.next(); const entry = entries.next(); document.getElementById('out').textContent = String(sheets.length) + ':' + String(key.value) + ':' + String(value.value) + ':' + String(entry.value.index) + ':' + String(entry.value.value);",
            "inline-script",
            &mut host,
        )
        .expect("document.styleSheets iterator helpers should resolve");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(3))
            .map(String::as_str),
        Some("2:0:[object CSSStyleSheet]:0:[object CSSStyleSheet]")
    );
    assert_eq!(host.document_style_sheets_items_calls, 4);
}

#[test]
fn runtime_resolves_document_style_sheets_named_item_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("first-style", ElementHandle::new(1), "First");
    host.seed_element("first-link", ElementHandle::new(2), "Second");
    host.seed_element("out", ElementHandle::new(3), "");
    host.seed_document_style_sheets_items(vec![ElementHandle::new(1), ElementHandle::new(2)]);
    host.seed_document_style_sheets_named_item("first-style", Some(ElementHandle::new(1)));
    host.seed_document_style_sheets_named_item("missing", None);

    runtime
        .eval_program(
            "const sheets = document.styleSheets; const first = sheets.namedItem('first-style'); document.getElementById('out').textContent = String(sheets.length) + ':' + String(first) + ':' + String(sheets.namedItem('missing'));",
            "inline-script",
            &mut host,
        )
        .expect("document.styleSheets namedItem should resolve");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(3))
            .map(String::as_str),
        Some("2:[object CSSStyleSheet]:null")
    );
    assert_eq!(
        host.document_style_sheets_named_item_calls,
        vec!["first-style".to_string(), "missing".to_string()]
    );
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
fn runtime_resolves_html_collection_named_property_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("root", ElementHandle::new(1), "");
    host.seed_element("first", ElementHandle::new(2), "First");
    host.seed_element("signup", ElementHandle::new(3), "");
    host.seed_element("mode-a", ElementHandle::new(4), "a");
    host.seed_element("mode-b", ElementHandle::new(5), "b");
    host.seed_element("out", ElementHandle::new(6), "");
    host.seed_html_collection_named_item(
        ElementHandle::new(1),
        "first",
        Some(ElementHandle::new(2)),
    );
    host.seed_html_collection_named_item(ElementHandle::new(1), "missing", None);
    host.seed_html_collection_form_elements_items(
        ElementHandle::new(3),
        vec![ElementHandle::new(4), ElementHandle::new(5)],
    );
    host.seed_html_collection_form_elements_named_items(
        ElementHandle::new(3),
        "mode",
        vec![ElementHandle::new(4), ElementHandle::new(5)],
    );

    runtime
        .eval_program(
            "const children = document.getElementById('root').children; const mode = document.getElementById('signup').elements.mode; document.getElementById('out').textContent = children.first.textContent + ':' + String(children.missing) + ':' + String(mode.length) + ':' + mode.item(0).value + ':' + mode.item(1).value + ':' + String(mode);",
            "inline-script",
            &mut host,
        )
        .expect("HTMLCollection legacy named property access should resolve");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(6))
            .map(String::as_str),
        Some("First:undefined:2:a:b:[object RadioNodeList]")
    );
    assert_eq!(
        host.html_collection_named_item_calls,
        vec![
            (ElementHandle::new(1), "first".to_string()),
            (ElementHandle::new(1), "missing".to_string()),
        ]
    );
    assert_eq!(
        host.html_collection_form_elements_named_items_calls,
        vec![
            (ElementHandle::new(3), "mode".to_string()),
            (ElementHandle::new(3), "mode".to_string()),
            (ElementHandle::new(3), "mode".to_string()),
            (ElementHandle::new(3), "mode".to_string()),
        ]
    );
}

#[test]
fn runtime_rejects_html_collection_reserved_named_property_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("root", ElementHandle::new(1), "");

    let error = runtime
        .eval_program(
            "document.getElementById('root').children.item;",
            "inline-script",
            &mut host,
        )
        .expect_err("reserved HTMLCollection named property access should fail");

    assert!(error.to_string().contains("unsupported member access"));
    assert!(error.to_string().contains("`item`"));
    assert!(error.to_string().contains("html collection value"));
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
fn runtime_supports_collection_entries_helpers() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("root", ElementHandle::new(1), "");
    host.seed_element("first", ElementHandle::new(2), "One");
    host.seed_element("second", ElementHandle::new(3), "Two");
    host.seed_element("out", ElementHandle::new(4), "");
    host.seed_node_child_nodes_items(
        HtmlCollectionScope::Document,
        vec![NodeHandle::new(10), NodeHandle::new(11)],
    );
    host.seed_node_name(NodeHandle::new(10), "main");
    host.seed_node_type(NodeHandle::new(10), 1);
    host.seed_node_text_content(NodeHandle::new(10), "");
    host.seed_node_name(NodeHandle::new(11), "div");
    host.seed_node_type(NodeHandle::new(11), 1);
    host.seed_node_text_content(NodeHandle::new(11), "");
    host.seed_element_children(
        ElementHandle::new(1),
        vec![ElementHandle::new(2), ElementHandle::new(3)],
    );

    runtime
        .eval_program(
            "const docEntries = document.childNodes.entries(); const childEntries = document.getElementById('root').children.entries(); const firstDoc = docEntries.next(); const secondDoc = docEntries.next(); const thirdDoc = docEntries.next(); const firstChild = childEntries.next(); const secondChild = childEntries.next(); const thirdChild = childEntries.next(); document.getElementById('out').textContent = String(firstDoc.value.index) + ':' + firstDoc.value.value.nodeName + ':' + String(secondDoc.value.index) + ':' + secondDoc.value.value.nodeName + ':' + String(thirdDoc.done) + ':' + String(firstChild.value.index) + ':' + firstChild.value.value.textContent + ':' + String(secondChild.value.index) + ':' + secondChild.value.value.textContent + ':' + String(thirdChild.done);",
            "inline-script",
            &mut host,
        )
        .expect("collection entries helpers should dispatch through the script runtime");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(4))
            .map(String::as_str),
        Some("0:main:1:div:true:0:One:1:Two:true")
    );
    assert_eq!(
        host.node_child_nodes_items_calls,
        vec![HtmlCollectionScope::Document]
    );
    assert_eq!(host.element_children_calls, vec![ElementHandle::new(1)]);
}

#[test]
fn runtime_supports_collection_to_string_helpers() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(1), "");

    runtime
        .eval_program(
            "document.getElementById('out').textContent = document.childNodes.toString() + ':' + window.navigator.plugins.toString();",
            "inline-script",
            &mut host,
        )
        .expect("collection toString helpers should dispatch through the script runtime");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(1))
            .map(String::as_str),
        Some("[object NodeList]:[object HTMLCollection]")
    );
}

#[test]
fn runtime_rejects_collection_to_string_arguments() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(1), "");

    let error = runtime
        .eval_program(
            "document.childNodes.toString(1);",
            "inline-script",
            &mut host,
        )
        .expect_err("collection toString helpers should reject arguments");

    assert!(
        error
            .to_string()
            .contains("NodeList.toString() expects no arguments")
    );
}

#[test]
fn runtime_supports_storage_accessors_and_methods() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(1), "");
    host.seed_local_storage("token", "abc");
    host.seed_session_storage("scratch", "seed");

    runtime
        .eval_program(
            "const local = window.localStorage; const session = document.defaultView.sessionStorage; const before = String(local) + ':' + String(session) + ':' + String(local.length) + ':' + String(session.length); const token = local.token; local.theme = 'dark'; local.removeItem('token'); session.scratch = 'xyz'; const sessionNamed = session.scratch; const sessionKey = session.key(0); session.clear(); document.getElementById('out').textContent = before + '|' + token + ':' + local.theme + ':' + sessionNamed + ':' + String(local.length) + ':' + String(local.key(0)) + ':' + String(session.length) + ':' + String(sessionKey);",
            "inline-script",
            &mut host,
        )
        .expect("storage accessors and methods should dispatch through the script runtime");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(1))
            .map(String::as_str),
        Some("[object Storage]:[object Storage]:1:1|abc:dark:xyz:1:theme:0:scratch")
    );
    assert_eq!(host.local_storage.get("token").map(String::as_str), None);
    assert_eq!(
        host.local_storage.get("theme").map(String::as_str),
        Some("dark")
    );
    assert!(host.session_storage.is_empty());
}

#[test]
fn runtime_rejects_storage_reserved_property_assignment() {
    let mut runtime = ScriptRuntime::new();
    let mut host = NoopHost::default();

    let error = runtime
        .eval_program(
            "window.localStorage.length = '2';",
            "inline-script",
            &mut host,
        )
        .expect_err("storage length should be read-only");

    assert!(
        error
            .to_string()
            .contains("cannot assign to `length` on storage value")
    );
}

#[test]
fn runtime_rejects_storage_method_wrong_arity() {
    let mut runtime = ScriptRuntime::new();
    let mut host = NoopHost::default();

    let error = runtime
        .eval_program(
            "window.localStorage.setItem('token');",
            "inline-script",
            &mut host,
        )
        .expect_err("storage methods should validate arity");

    assert!(
        error
            .to_string()
            .contains("setItem() expects exactly two arguments")
    );
}

#[test]
fn runtime_rejects_document_create_element_wrong_arity() {
    let mut runtime = ScriptRuntime::new();
    let mut host = NoopHost::default();

    let error = runtime
        .eval_program("document.createElement();", "inline-script", &mut host)
        .expect_err("createElement should validate arity");

    assert!(
        error
            .to_string()
            .contains("document.createElement() expects exactly one argument")
    );
}

#[test]
fn runtime_rejects_document_create_text_node_wrong_arity() {
    let mut runtime = ScriptRuntime::new();
    let mut host = NoopHost::default();

    let error = runtime
        .eval_program("document.createTextNode();", "inline-script", &mut host)
        .expect_err("createTextNode should validate arity");

    assert!(
        error
            .to_string()
            .contains("document.createTextNode() expects exactly one argument")
    );
}

#[test]
fn runtime_rejects_document_create_comment_wrong_arity() {
    let mut runtime = ScriptRuntime::new();
    let mut host = NoopHost::default();

    let error = runtime
        .eval_program("document.createComment();", "inline-script", &mut host)
        .expect_err("createComment should validate arity");

    assert!(
        error
            .to_string()
            .contains("document.createComment() expects exactly one argument")
    );
}

#[test]
fn runtime_rejects_document_create_document_fragment_wrong_arity() {
    let mut runtime = ScriptRuntime::new();
    let mut host = NoopHost::default();

    let error = runtime
        .eval_program(
            "document.createDocumentFragment('unexpected');",
            "inline-script",
            &mut host,
        )
        .expect_err("createDocumentFragment should validate arity");

    assert!(
        error
            .to_string()
            .contains("document.createDocumentFragment() expects no arguments")
    );
}

#[test]
fn runtime_resolves_document_create_document_fragment_access() {
    struct FragmentHost {
        create_element_calls: Vec<String>,
        create_text_node_calls: Vec<String>,
        fragment_children: Vec<NodeHandle>,
        append_child_calls: Vec<(ElementHandle, NodeHandle)>,
        append_calls: Vec<(ElementHandle, Vec<NodeHandle>)>,
    }

    impl HostBindings for FragmentHost {
        fn document_create_element(&mut self, tag_name: &str) -> bt_script::Result<ElementHandle> {
            self.create_element_calls.push(tag_name.to_string());
            Ok(match tag_name {
                "div" => ElementHandle::new(3),
                "template" => ElementHandle::new(4),
                _ => ElementHandle::new(99),
            })
        }

        fn document_create_text_node(&mut self, text: &str) -> bt_script::Result<NodeHandle> {
            self.create_text_node_calls.push(text.to_string());
            Ok(NodeHandle::new(10))
        }

        fn node_child_nodes_items(
            &mut self,
            scope: HtmlCollectionScope,
        ) -> bt_script::Result<Vec<NodeHandle>> {
            Ok(match scope {
                HtmlCollectionScope::Element(element) if element.raw() == 4 => {
                    self.fragment_children.clone()
                }
                _ => Vec::new(),
            })
        }

        fn node_type(&mut self, node: NodeHandle) -> bt_script::Result<u8> {
            Ok(match node.raw() {
                10 => 3,
                _ => 1,
            })
        }

        fn element_append_child(
            &mut self,
            parent: ElementHandle,
            child: NodeHandle,
        ) -> bt_script::Result<()> {
            self.append_child_calls.push((parent, child));
            if parent.raw() == 4 {
                self.fragment_children.push(child);
            }
            Ok(())
        }

        fn element_append(
            &mut self,
            element: ElementHandle,
            children: Vec<NodeHandle>,
        ) -> bt_script::Result<()> {
            self.append_calls.push((element, children));
            Ok(())
        }
    }

    let mut runtime = ScriptRuntime::new();
    let mut host = FragmentHost {
        create_element_calls: Vec::new(),
        create_text_node_calls: Vec::new(),
        fragment_children: Vec::new(),
        append_child_calls: Vec::new(),
        append_calls: Vec::new(),
    };

    runtime
        .eval_program(
            "const root = document.createElement('div'); const frag = document.createDocumentFragment(); frag.appendChild(document.createTextNode('Hello')); root.appendChild(frag);",
            "inline-script",
            &mut host,
        )
        .expect("document.createDocumentFragment should resolve");

    assert_eq!(host.create_element_calls, vec!["div", "template"]);
    assert_eq!(host.create_text_node_calls, vec!["Hello"]);
    assert_eq!(
        host.append_child_calls,
        vec![(ElementHandle::new(4), NodeHandle::new(10))]
    );
    assert_eq!(
        host.append_calls,
        vec![(ElementHandle::new(3), vec![NodeHandle::new(10)])]
    );
}

#[test]
fn runtime_rejects_document_import_node_wrong_arity() {
    let mut runtime = ScriptRuntime::new();
    let mut host = NoopHost::default();

    let error = runtime
        .eval_program("document.importNode();", "inline-script", &mut host)
        .expect_err("importNode should validate arity");

    assert!(
        error
            .to_string()
            .contains("document.importNode() expects one or two arguments")
    );
}

#[test]
fn runtime_rejects_document_import_node_invalid_argument() {
    let mut runtime = ScriptRuntime::new();
    let mut host = NoopHost::default();

    let error = runtime
        .eval_program("document.importNode(document);", "inline-script", &mut host)
        .expect_err("importNode should reject non-node arguments");

    assert!(
        error
            .to_string()
            .contains("document.importNode() expects a node or DocumentFragment argument")
    );
}

#[test]
fn runtime_resolves_document_import_node_access() {
    struct ImportHost {
        create_element_calls: Vec<String>,
        create_text_node_calls: Vec<String>,
        fragment_children: BTreeMap<u64, Vec<NodeHandle>>,
        clone_calls: Vec<(NodeHandle, bool)>,
        append_calls: Vec<(ElementHandle, Vec<NodeHandle>)>,
    }

    impl HostBindings for ImportHost {
        fn document_create_element(&mut self, tag_name: &str) -> bt_script::Result<ElementHandle> {
            self.create_element_calls.push(tag_name.to_string());
            Ok(match tag_name {
                "div" => ElementHandle::new(3),
                "template" => ElementHandle::new(4),
                _ => ElementHandle::new(99),
            })
        }

        fn document_create_text_node(&mut self, text: &str) -> bt_script::Result<NodeHandle> {
            self.create_text_node_calls.push(text.to_string());
            Ok(NodeHandle::new(10))
        }

        fn node_child_nodes_items(
            &mut self,
            scope: HtmlCollectionScope,
        ) -> bt_script::Result<Vec<NodeHandle>> {
            Ok(match scope {
                HtmlCollectionScope::Element(element) => self
                    .fragment_children
                    .get(&element.raw())
                    .cloned()
                    .unwrap_or_default(),
                _ => Vec::new(),
            })
        }

        fn node_clone(&mut self, node: NodeHandle, deep: bool) -> bt_script::Result<NodeHandle> {
            self.clone_calls.push((node, deep));
            if node.raw() == 4 {
                let cloned = NodeHandle::new(40);
                let children = if deep {
                    self.fragment_children
                        .get(&node.raw())
                        .cloned()
                        .unwrap_or_default()
                } else {
                    Vec::new()
                };
                self.fragment_children.insert(cloned.raw(), children);
                Ok(cloned)
            } else {
                Ok(NodeHandle::new(node.raw() + 1))
            }
        }

        fn node_type(&mut self, node: NodeHandle) -> bt_script::Result<u8> {
            Ok(match node.raw() {
                10 => 3,
                4 | 40 => 1,
                _ => 1,
            })
        }

        fn element_append_child(
            &mut self,
            parent: ElementHandle,
            child: NodeHandle,
        ) -> bt_script::Result<()> {
            if parent.raw() == 4 {
                self.fragment_children
                    .entry(parent.raw())
                    .or_default()
                    .push(child);
            }
            Ok(())
        }

        fn element_append(
            &mut self,
            element: ElementHandle,
            children: Vec<NodeHandle>,
        ) -> bt_script::Result<()> {
            self.append_calls.push((element, children));
            Ok(())
        }
    }

    let mut runtime = ScriptRuntime::new();
    let mut host = ImportHost {
        create_element_calls: Vec::new(),
        create_text_node_calls: Vec::new(),
        fragment_children: BTreeMap::new(),
        clone_calls: Vec::new(),
        append_calls: Vec::new(),
    };

    runtime
        .eval_program(
            "const source = document.createDocumentFragment(); source.appendChild(document.createTextNode('Hello')); const imported = document.importNode(source, true); const root = document.createElement('div'); root.appendChild(imported);",
            "inline-script",
            &mut host,
        )
        .expect("document.importNode should resolve");

    assert_eq!(host.create_element_calls, vec!["template", "div"]);
    assert_eq!(host.create_text_node_calls, vec!["Hello"]);
    assert_eq!(host.clone_calls, vec![(NodeHandle::new(4), true)]);
    assert_eq!(
        host.append_calls,
        vec![(ElementHandle::new(3), vec![NodeHandle::new(10)])]
    );
}

#[test]
fn runtime_rejects_document_normalize_wrong_arity() {
    let mut runtime = ScriptRuntime::new();
    let mut host = NoopHost::default();

    let error = runtime
        .eval_program(
            "document.normalize('unexpected');",
            "inline-script",
            &mut host,
        )
        .expect_err("Document.normalize should validate arity");

    assert!(
        error
            .to_string()
            .contains("normalize() expects no arguments")
    );
}

#[test]
fn runtime_resolves_document_normalize_access() {
    struct NormalizeHost {
        document_normalize_calls: usize,
    }

    impl HostBindings for NormalizeHost {
        fn document_normalize(&mut self) -> bt_script::Result<()> {
            self.document_normalize_calls += 1;
            Ok(())
        }
    }

    let mut runtime = ScriptRuntime::new();
    let mut host = NormalizeHost {
        document_normalize_calls: 0,
    };

    runtime
        .eval_program("document.normalize();", "inline-script", &mut host)
        .expect("Document.normalize should resolve through the script runtime");

    assert_eq!(host.document_normalize_calls, 1);
}

#[test]
fn runtime_resolves_node_normalize_access() {
    struct NormalizeHost {
        element: ElementHandle,
        node_normalize_calls: Vec<NodeHandle>,
    }

    impl HostBindings for NormalizeHost {
        fn document_get_element_by_id(
            &mut self,
            id: &str,
        ) -> bt_script::Result<Option<ElementHandle>> {
            Ok(match id {
                "root" => Some(self.element),
                _ => None,
            })
        }

        fn node_normalize(&mut self, node: NodeHandle) -> bt_script::Result<()> {
            self.node_normalize_calls.push(node);
            Ok(())
        }
    }

    let mut runtime = ScriptRuntime::new();
    let mut host = NormalizeHost {
        element: ElementHandle::new(11),
        node_normalize_calls: Vec::new(),
    };

    runtime
        .eval_program(
            "document.getElementById('root').normalize();",
            "inline-script",
            &mut host,
        )
        .expect("Node.normalize should resolve through the script runtime");

    assert_eq!(host.node_normalize_calls, vec![NodeHandle::new(11)]);
}

#[test]
fn runtime_rejects_node_before_non_node_argument() {
    struct BeforeHost;

    impl HostBindings for BeforeHost {
        fn document_get_element_by_id(
            &mut self,
            id: &str,
        ) -> bt_script::Result<Option<ElementHandle>> {
            Ok(match id {
                "root" => Some(ElementHandle::new(11)),
                _ => None,
            })
        }

        fn document_create_text_node(&mut self, _text: &str) -> bt_script::Result<NodeHandle> {
            Ok(NodeHandle::new(12))
        }

        fn element_append_child(
            &mut self,
            _parent: ElementHandle,
            _child: NodeHandle,
        ) -> bt_script::Result<()> {
            Ok(())
        }

        fn node_parent(&mut self, _node: NodeHandle) -> bt_script::Result<Option<NodeHandle>> {
            Ok(Some(NodeHandle::new(11)))
        }

        fn node_type(&mut self, node: NodeHandle) -> bt_script::Result<u8> {
            Ok(match node.raw() {
                11 => 1,
                _ => 3,
            })
        }
    }

    let mut runtime = ScriptRuntime::new();
    let mut host = BeforeHost;

    let error = runtime
        .eval_program(
            "const root = document.getElementById('root'); const child = document.createTextNode('Hello'); root.appendChild(child); child.before('unexpected');",
            "inline-script",
            &mut host,
        )
        .expect_err("Node.before should validate its mutation arguments");

    assert!(
        error
            .to_string()
            .contains("before() expects node or DocumentFragment arguments")
    );
}

#[test]
fn runtime_resolves_node_before_and_after_access() {
    struct BeforeAfterHost {
        next_node_handle: u64,
        node_parent_results: BTreeMap<NodeHandle, Option<NodeHandle>>,
        node_before_calls: Vec<(NodeHandle, Vec<NodeHandle>)>,
        node_after_calls: Vec<(NodeHandle, Vec<NodeHandle>)>,
    }

    impl HostBindings for BeforeAfterHost {
        fn document_get_element_by_id(
            &mut self,
            id: &str,
        ) -> bt_script::Result<Option<ElementHandle>> {
            Ok(match id {
                "root" => Some(ElementHandle::new(11)),
                _ => None,
            })
        }

        fn document_create_text_node(&mut self, _text: &str) -> bt_script::Result<NodeHandle> {
            let handle = NodeHandle::new(self.next_node_handle);
            self.next_node_handle += 1;
            Ok(handle)
        }

        fn element_append_child(
            &mut self,
            parent: ElementHandle,
            child: NodeHandle,
        ) -> bt_script::Result<()> {
            self.node_parent_results
                .insert(child, Some(NodeHandle::new(parent.raw())));
            Ok(())
        }

        fn node_before(
            &mut self,
            node: NodeHandle,
            children: Vec<NodeHandle>,
        ) -> bt_script::Result<()> {
            self.node_before_calls.push((node, children));
            Ok(())
        }

        fn node_after(
            &mut self,
            node: NodeHandle,
            children: Vec<NodeHandle>,
        ) -> bt_script::Result<()> {
            self.node_after_calls.push((node, children));
            Ok(())
        }

        fn node_parent(&mut self, node: NodeHandle) -> bt_script::Result<Option<NodeHandle>> {
            Ok(self.node_parent_results.get(&node).copied().unwrap_or(None))
        }

        fn node_type(&mut self, node: NodeHandle) -> bt_script::Result<u8> {
            Ok(match node.raw() {
                11 => 1,
                _ => 3,
            })
        }
    }

    let mut runtime = ScriptRuntime::new();
    let mut host = BeforeAfterHost {
        next_node_handle: 12,
        node_parent_results: BTreeMap::new(),
        node_before_calls: Vec::new(),
        node_after_calls: Vec::new(),
    };

    runtime
        .eval_program(
            "const root = document.getElementById('root'); const child = document.createTextNode('Hello'); root.appendChild(child); child.before(document.createTextNode('Before')); child.after(document.createTextNode('After'));",
            "inline-script",
            &mut host,
        )
        .expect("Node.before and Node.after should resolve through the script runtime");

    assert_eq!(
        host.node_before_calls,
        vec![(NodeHandle::new(12), vec![NodeHandle::new(13)])]
    );
    assert_eq!(
        host.node_after_calls,
        vec![(NodeHandle::new(12), vec![NodeHandle::new(14)])]
    );
}

#[test]
fn runtime_rejects_compare_document_position_non_node_argument() {
    let mut runtime = ScriptRuntime::new();
    let mut host = NoopHost::default();

    let error = runtime
        .eval_program(
            "document.compareDocumentPosition('unexpected');",
            "inline-script",
            &mut host,
        )
        .expect_err("compareDocumentPosition should validate its argument");

    assert!(
        error
            .to_string()
            .contains("compareDocumentPosition() expects a node argument")
    );
}

#[test]
fn runtime_resolves_compare_document_position_access() {
    struct CompareHost {
        compare_calls: Vec<(NodeHandle, NodeHandle)>,
    }

    impl HostBindings for CompareHost {
        fn document_get_element_by_id(
            &mut self,
            id: &str,
        ) -> bt_script::Result<Option<ElementHandle>> {
            Ok(match id {
                "root" => Some(ElementHandle::new(11)),
                _ => None,
            })
        }

        fn node_compare_document_position(
            &mut self,
            node: NodeHandle,
            other: NodeHandle,
        ) -> bt_script::Result<u16> {
            self.compare_calls.push((node, other));
            Ok(match (node.raw(), other.raw()) {
                (0, 11) => 20,
                (11, 0) => 10,
                _ => 0,
            })
        }

        fn node_type(&mut self, node: NodeHandle) -> bt_script::Result<u8> {
            Ok(match node.raw() {
                11 => 1,
                _ => 3,
            })
        }
    }

    let mut runtime = ScriptRuntime::new();
    let mut host = CompareHost {
        compare_calls: Vec::new(),
    };

    runtime
        .eval_program(
            "const root = document.getElementById('root'); document.compareDocumentPosition(root); root.compareDocumentPosition(document);",
            "inline-script",
            &mut host,
        )
        .expect("compareDocumentPosition should resolve through the script runtime");

    assert_eq!(
        host.compare_calls,
        vec![
            (NodeHandle::new(0), NodeHandle::new(11)),
            (NodeHandle::new(11), NodeHandle::new(0))
        ]
    );
}

#[test]
fn runtime_rejects_is_same_node_wrong_arity() {
    let mut runtime = ScriptRuntime::new();
    let mut host = NoopHost::default();

    let error = runtime
        .eval_program("document.isSameNode();", "inline-script", &mut host)
        .expect_err("isSameNode should validate arity");

    assert!(
        error
            .message()
            .contains("isSameNode() expects exactly one argument")
    );
}

#[test]
fn runtime_resolves_is_same_node_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(0), "");

    runtime
        .eval_program(
            "document.getElementById('out').textContent = String(document.isSameNode(document)) + ':' + String(document.isSameNode(null));",
            "inline-script",
            &mut host,
        )
        .expect("isSameNode should resolve through the script runtime");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(0))
            .map(String::as_str),
        Some("true:false")
    );
}

#[test]
fn runtime_rejects_is_equal_node_non_node_argument() {
    let mut runtime = ScriptRuntime::new();
    let mut host = NoopHost::default();

    let error = runtime
        .eval_program(
            "document.isEqualNode('unexpected');",
            "inline-script",
            &mut host,
        )
        .expect_err("isEqualNode should reject non-node arguments");

    assert!(
        error
            .message()
            .contains("isEqualNode() expects a node or null reference")
    );
}

#[test]
fn runtime_resolves_is_equal_node_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("root", ElementHandle::new(11), "Root");
    host.seed_element("mirror", ElementHandle::new(12), "Mirror");
    host.seed_element("tpl", ElementHandle::new(21), "");
    host.seed_element("other", ElementHandle::new(22), "");
    host.seed_element("out", ElementHandle::new(0), "");
    host.element_tag_name_results
        .insert(ElementHandle::new(21), "template".to_string());
    host.element_tag_name_results
        .insert(ElementHandle::new(22), "template".to_string());
    host.node_is_equal_node_results
        .insert((NodeHandle::new(11), NodeHandle::new(12)), true);
    host.template_content_is_equal_node_results
        .insert((ElementHandle::new(21), ElementHandle::new(22)), true);

    runtime
        .eval_program(
            "const root = document.getElementById('root'); const mirror = document.getElementById('mirror'); const tpl = document.getElementById('tpl'); const other = document.getElementById('other'); document.getElementById('out').textContent = String(root.isSameNode(root)) + ':' + String(root.isSameNode(mirror)) + ':' + String(root.isEqualNode(mirror)) + ':' + String(tpl.content.isSameNode(other.content)) + ':' + String(tpl.content.isEqualNode(other.content)) + ':' + String(root.isEqualNode(null));",
            "inline-script",
            &mut host,
        )
        .expect("isEqualNode should resolve through the script runtime");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(0))
            .map(String::as_str),
        Some("true:false:true:false:true:false")
    );
    assert_eq!(
        host.node_is_equal_node_calls,
        vec![(NodeHandle::new(11), NodeHandle::new(12))]
    );
    assert_eq!(
        host.template_content_is_equal_node_calls,
        vec![(ElementHandle::new(21), ElementHandle::new(22))]
    );
}

#[test]
fn runtime_rejects_node_remove_wrong_arity() {
    struct RemoveHost;

    impl HostBindings for RemoveHost {
        fn document_create_text_node(&mut self, _text: &str) -> bt_script::Result<NodeHandle> {
            Ok(NodeHandle::new(12))
        }

        fn node_type(&mut self, node: NodeHandle) -> bt_script::Result<u8> {
            Ok(match node.raw() {
                12 => 3,
                _ => 1,
            })
        }
    }

    let mut runtime = ScriptRuntime::new();
    let mut host = RemoveHost;

    let error = runtime
        .eval_program(
            "document.createTextNode('Hello').remove('unexpected');",
            "inline-script",
            &mut host,
        )
        .expect_err("Node.remove should validate arity");

    assert!(error.to_string().contains("remove() expects no arguments"));
}

#[test]
fn runtime_resolves_node_remove_access() {
    struct RemoveHost {
        node_parent_results: BTreeMap<NodeHandle, Option<NodeHandle>>,
        node_replace_with_calls: Vec<(NodeHandle, Vec<NodeHandle>)>,
    }

    impl HostBindings for RemoveHost {
        fn document_get_element_by_id(
            &mut self,
            id: &str,
        ) -> bt_script::Result<Option<ElementHandle>> {
            Ok(match id {
                "root" => Some(ElementHandle::new(11)),
                _ => None,
            })
        }

        fn document_create_element(&mut self, _tag_name: &str) -> bt_script::Result<ElementHandle> {
            Ok(ElementHandle::new(11))
        }

        fn document_create_text_node(&mut self, _text: &str) -> bt_script::Result<NodeHandle> {
            Ok(NodeHandle::new(12))
        }

        fn element_append_child(
            &mut self,
            _parent: ElementHandle,
            _child: NodeHandle,
        ) -> bt_script::Result<()> {
            Ok(())
        }

        fn node_parent(&mut self, node: NodeHandle) -> bt_script::Result<Option<NodeHandle>> {
            Ok(self.node_parent_results.get(&node).copied().unwrap_or(None))
        }

        fn node_replace_with(
            &mut self,
            node: NodeHandle,
            children: Vec<NodeHandle>,
        ) -> bt_script::Result<()> {
            self.node_replace_with_calls.push((node, children));
            self.node_parent_results.insert(node, None);
            Ok(())
        }

        fn node_type(&mut self, node: NodeHandle) -> bt_script::Result<u8> {
            Ok(match node.raw() {
                12 => 3,
                _ => 1,
            })
        }
    }

    let mut runtime = ScriptRuntime::new();
    let mut host = RemoveHost {
        node_parent_results: BTreeMap::from([(NodeHandle::new(12), Some(NodeHandle::new(11)))]),
        node_replace_with_calls: Vec::new(),
    };

    runtime
        .eval_program(
            "const root = document.getElementById('root'); const child = document.createTextNode('Hello'); root.appendChild(child); child.remove();",
            "inline-script",
            &mut host,
        )
        .expect("Node.remove should resolve through the script runtime");

    assert_eq!(
        host.node_replace_with_calls,
        vec![(NodeHandle::new(12), Vec::new())]
    );
}

#[test]
fn runtime_rejects_clone_node_wrong_arity() {
    struct CloneHost;

    impl HostBindings for CloneHost {
        fn document_create_element(&mut self, _tag_name: &str) -> bt_script::Result<ElementHandle> {
            Ok(ElementHandle::new(1))
        }

        fn node_type(&mut self, _node: NodeHandle) -> bt_script::Result<u8> {
            Ok(1)
        }
    }

    let mut runtime = ScriptRuntime::new();
    let mut host = CloneHost;

    let error = runtime
        .eval_program(
            "const root = document.createElement('div'); root.cloneNode(true, false);",
            "inline-script",
            &mut host,
        )
        .expect_err("cloneNode should validate arity");

    assert!(
        error
            .to_string()
            .contains("cloneNode() expects at most one argument")
    );
}

#[test]
fn runtime_resolves_node_replace_with_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("first", ElementHandle::new(11), "First");
    host.seed_element("second", ElementHandle::new(12), "Second");

    runtime
        .eval_program(
            "document.getElementById('first').replaceWith(document.getElementById('second'));",
            "inline-script",
            &mut host,
        )
        .expect("Element.replaceWith should resolve through the script runtime");

    assert_eq!(
        host.node_replace_with_calls,
        vec![(NodeHandle::new(11), vec![NodeHandle::new(12)])]
    );
}

#[test]
fn runtime_resolves_node_contains_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("root", ElementHandle::new(11), "Root");
    host.seed_element("child", ElementHandle::new(12), "Child");

    runtime
        .eval_program(
            "document.getElementById('root').contains(document.getElementById('child')); document.contains(document.getElementById('child'));",
            "inline-script",
            &mut host,
        )
        .expect("contains should resolve through the script runtime");

    assert_eq!(
        host.node_contains_calls,
        vec![(NodeHandle::new(11), NodeHandle::new(12))]
    );
    assert_eq!(host.document_contains_calls, vec![NodeHandle::new(12)]);
}

#[test]
fn runtime_resolves_node_has_child_nodes_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(0), "");
    host.seed_element("root", ElementHandle::new(21), "Root");
    host.seed_element("child", ElementHandle::new(22), "Child");
    host.document_has_child_nodes_result = true;
    host.node_has_child_nodes_results
        .insert(NodeHandle::new(21), true);
    host.node_has_child_nodes_results
        .insert(NodeHandle::new(22), false);

    runtime
        .eval_program(
            "document.getElementById('out').textContent = String(document.hasChildNodes()) + ':' + String(document.getElementById('root').hasChildNodes()) + ':' + String(document.getElementById('child').hasChildNodes());",
            "inline-script",
            &mut host,
        )
        .expect("hasChildNodes should resolve through the script runtime");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(0))
            .map(String::as_str),
        Some("true:true:false")
    );
    assert_eq!(host.document_has_child_nodes_calls, 1);
    assert_eq!(
        host.node_has_child_nodes_calls,
        vec![NodeHandle::new(21), NodeHandle::new(22)]
    );
}

#[test]
fn runtime_resolves_first_and_last_child_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(0), "");
    host.seed_element("tpl", ElementHandle::new(6), "");
    host.seed_element_tag_name(ElementHandle::new(6), "template");
    host.seed_document_document_element(Some(ElementHandle::new(1)));
    host.seed_document_body(Some(ElementHandle::new(3)));
    host.seed_node_child_nodes_items(
        HtmlCollectionScope::Document,
        vec![NodeHandle::new(9), NodeHandle::new(1)],
    );
    host.seed_node_child_nodes_items(
        HtmlCollectionScope::Element(ElementHandle::new(1)),
        vec![NodeHandle::new(2), NodeHandle::new(3)],
    );
    host.seed_node_child_nodes_items(
        HtmlCollectionScope::Element(ElementHandle::new(3)),
        vec![NodeHandle::new(4), NodeHandle::new(5)],
    );
    host.seed_node_child_nodes_items(HtmlCollectionScope::Node(NodeHandle::new(4)), vec![]);
    host.seed_node_child_nodes_items(
        HtmlCollectionScope::Element(ElementHandle::new(6)),
        vec![NodeHandle::new(7), NodeHandle::new(8)],
    );
    host.seed_node_type(NodeHandle::new(9), 8);
    host.seed_node_type(NodeHandle::new(1), 1);
    host.seed_node_type(NodeHandle::new(2), 1);
    host.seed_node_type(NodeHandle::new(3), 1);
    host.seed_node_type(NodeHandle::new(4), 3);
    host.seed_node_type(NodeHandle::new(5), 8);
    host.seed_node_type(NodeHandle::new(7), 1);
    host.seed_node_type(NodeHandle::new(8), 8);

    runtime
        .eval_program(
            "const html = document.documentElement; const body = document.body; const tpl = document.getElementById('tpl').content; const text = body.childNodes.item(0); document.getElementById('out').textContent = String(document.firstChild) + ':' + String(document.lastChild) + ':' + String(html.firstChild) + ':' + String(html.lastChild) + ':' + String(body.firstChild) + ':' + String(body.lastChild) + ':' + String(text.firstChild) + ':' + String(text.lastChild) + ':' + String(tpl.firstChild) + ':' + String(tpl.lastChild);",
            "inline-script",
            &mut host,
        )
        .expect("firstChild and lastChild should resolve through the script runtime");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(0))
            .map(String::as_str),
        Some(
            "[object Node]:[object Element]:[object Element]:[object Element]:[object Node]:[object Node]:null:null:[object Element]:[object Node]"
        )
    );
    assert_eq!(
        host.node_child_nodes_items_calls,
        vec![
            HtmlCollectionScope::Element(ElementHandle::new(3)),
            HtmlCollectionScope::Document,
            HtmlCollectionScope::Document,
            HtmlCollectionScope::Element(ElementHandle::new(1)),
            HtmlCollectionScope::Element(ElementHandle::new(1)),
            HtmlCollectionScope::Element(ElementHandle::new(3)),
            HtmlCollectionScope::Element(ElementHandle::new(3)),
            HtmlCollectionScope::Node(NodeHandle::new(4)),
            HtmlCollectionScope::Node(NodeHandle::new(4)),
            HtmlCollectionScope::Element(ElementHandle::new(6)),
            HtmlCollectionScope::Element(ElementHandle::new(6)),
        ]
    );
}

#[test]
fn runtime_resolves_sibling_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(5), "");
    host.seed_element("tpl", ElementHandle::new(7), "");
    host.seed_element_tag_name(ElementHandle::new(7), "template");
    host.seed_document_document_element(Some(ElementHandle::new(1)));
    host.seed_document_head(Some(ElementHandle::new(2)));
    host.seed_document_body(Some(ElementHandle::new(3)));
    host.seed_node_child_nodes_items(
        HtmlCollectionScope::Document,
        vec![NodeHandle::new(11), NodeHandle::new(1)],
    );
    host.seed_node_child_nodes_items(
        HtmlCollectionScope::Element(ElementHandle::new(1)),
        vec![NodeHandle::new(2), NodeHandle::new(3)],
    );
    host.seed_node_child_nodes_items(
        HtmlCollectionScope::Element(ElementHandle::new(3)),
        vec![
            NodeHandle::new(4),
            NodeHandle::new(5),
            NodeHandle::new(6),
            NodeHandle::new(7),
            NodeHandle::new(8),
        ],
    );
    host.seed_node_child_nodes_items(
        HtmlCollectionScope::Element(ElementHandle::new(7)),
        vec![NodeHandle::new(9), NodeHandle::new(10)],
    );
    host.seed_node_parent(NodeHandle::new(11), Some(NodeHandle::new(0)));
    host.seed_node_parent(NodeHandle::new(1), Some(NodeHandle::new(0)));
    host.seed_node_parent(NodeHandle::new(2), Some(NodeHandle::new(1)));
    host.seed_node_parent(NodeHandle::new(3), Some(NodeHandle::new(1)));
    host.seed_node_parent(NodeHandle::new(4), Some(NodeHandle::new(3)));
    host.seed_node_parent(NodeHandle::new(5), Some(NodeHandle::new(3)));
    host.seed_node_parent(NodeHandle::new(6), Some(NodeHandle::new(3)));
    host.seed_node_parent(NodeHandle::new(7), Some(NodeHandle::new(3)));
    host.seed_node_parent(NodeHandle::new(8), Some(NodeHandle::new(3)));
    host.seed_node_parent(NodeHandle::new(9), Some(NodeHandle::new(7)));
    host.seed_node_parent(NodeHandle::new(10), Some(NodeHandle::new(7)));
    host.seed_node_type(NodeHandle::new(0), 9);
    host.seed_node_type(NodeHandle::new(1), 1);
    host.seed_node_type(NodeHandle::new(2), 1);
    host.seed_node_type(NodeHandle::new(3), 1);
    host.seed_node_type(NodeHandle::new(4), 3);
    host.seed_node_type(NodeHandle::new(5), 1);
    host.seed_node_type(NodeHandle::new(6), 1);
    host.seed_node_type(NodeHandle::new(7), 1);
    host.seed_node_type(NodeHandle::new(8), 1);
    host.seed_node_type(NodeHandle::new(9), 1);
    host.seed_node_type(NodeHandle::new(10), 8);
    host.seed_node_type(NodeHandle::new(11), 8);

    runtime
        .eval_program(
            "const html = document.documentElement; const head = document.head; const body = document.body; const tpl = document.getElementById('tpl'); const content = tpl.content; const text = body.childNodes.item(0); const out = body.childNodes.item(1); document.getElementById('out').textContent = String(document.nextSibling) + ':' + String(document.previousSibling) + ':' + String(html.previousSibling) + ':' + String(head.nextSibling) + ':' + String(body.previousSibling) + ':' + String(body.nextSibling) + ':' + String(body.firstChild.nextSibling) + ':' + String(body.lastChild.previousSibling) + ':' + String(text.nextSibling) + ':' + String(out.previousSibling) + ':' + String(tpl.nextSibling) + ':' + String(tpl.previousSibling) + ':' + String(content.nextSibling) + ':' + String(content.previousSibling) + ':' + String(content.firstChild.nextSibling) + ':' + String(content.lastChild.previousSibling);",
            "inline-script",
            &mut host,
        )
        .expect("sibling reflection should resolve through the script runtime");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(5))
            .map(String::as_str),
        Some(
            "null:null:[object Node]:[object Element]:[object Element]:null:[object Element]:[object Element]:[object Element]:[object Node]:[object Element]:[object Element]:null:null:[object Node]:[object Element]"
        )
    );
    assert_eq!(
        host.node_child_nodes_items_calls,
        vec![
            HtmlCollectionScope::Element(ElementHandle::new(3)),
            HtmlCollectionScope::Element(ElementHandle::new(3)),
            HtmlCollectionScope::Document,
            HtmlCollectionScope::Element(ElementHandle::new(1)),
            HtmlCollectionScope::Element(ElementHandle::new(1)),
            HtmlCollectionScope::Element(ElementHandle::new(1)),
            HtmlCollectionScope::Element(ElementHandle::new(3)),
            HtmlCollectionScope::Element(ElementHandle::new(3)),
            HtmlCollectionScope::Element(ElementHandle::new(3)),
            HtmlCollectionScope::Element(ElementHandle::new(3)),
            HtmlCollectionScope::Element(ElementHandle::new(3)),
            HtmlCollectionScope::Element(ElementHandle::new(3)),
            HtmlCollectionScope::Element(ElementHandle::new(3)),
            HtmlCollectionScope::Element(ElementHandle::new(3)),
            HtmlCollectionScope::Element(ElementHandle::new(7)),
            HtmlCollectionScope::Element(ElementHandle::new(7)),
            HtmlCollectionScope::Element(ElementHandle::new(7)),
            HtmlCollectionScope::Element(ElementHandle::new(7)),
        ]
    );
}

#[test]
fn runtime_resolves_element_sibling_access() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(5), "");
    host.seed_document_document_element(Some(ElementHandle::new(1)));
    host.seed_document_head(Some(ElementHandle::new(2)));
    host.seed_document_body(Some(ElementHandle::new(3)));
    host.seed_node_child_nodes_items(
        HtmlCollectionScope::Document,
        vec![NodeHandle::new(11), NodeHandle::new(1)],
    );
    host.seed_node_child_nodes_items(
        HtmlCollectionScope::Element(ElementHandle::new(1)),
        vec![NodeHandle::new(2), NodeHandle::new(3)],
    );
    host.seed_node_child_nodes_items(
        HtmlCollectionScope::Element(ElementHandle::new(3)),
        vec![
            NodeHandle::new(4),
            NodeHandle::new(5),
            NodeHandle::new(6),
            NodeHandle::new(7),
        ],
    );
    host.seed_node_parent(NodeHandle::new(11), Some(NodeHandle::new(0)));
    host.seed_node_parent(NodeHandle::new(1), Some(NodeHandle::new(0)));
    host.seed_node_parent(NodeHandle::new(2), Some(NodeHandle::new(1)));
    host.seed_node_parent(NodeHandle::new(3), Some(NodeHandle::new(1)));
    host.seed_node_parent(NodeHandle::new(4), Some(NodeHandle::new(3)));
    host.seed_node_parent(NodeHandle::new(5), Some(NodeHandle::new(3)));
    host.seed_node_parent(NodeHandle::new(6), Some(NodeHandle::new(3)));
    host.seed_node_parent(NodeHandle::new(7), Some(NodeHandle::new(3)));
    host.seed_node_type(NodeHandle::new(0), 9);
    host.seed_node_type(NodeHandle::new(1), 1);
    host.seed_node_type(NodeHandle::new(2), 1);
    host.seed_node_type(NodeHandle::new(3), 1);
    host.seed_node_type(NodeHandle::new(4), 3);
    host.seed_node_type(NodeHandle::new(5), 1);
    host.seed_node_type(NodeHandle::new(6), 1);
    host.seed_node_type(NodeHandle::new(7), 1);
    host.seed_node_type(NodeHandle::new(11), 8);

    runtime
        .eval_program(
            "const html = document.documentElement; const head = document.head; const body = document.body; const text = body.firstChild; const out = body.childNodes.item(1); const main = body.childNodes.item(2); const script = body.lastChild; document.getElementById('out').textContent = String(document.nextElementSibling) + ':' + String(document.previousElementSibling) + ':' + String(html.nextElementSibling) + ':' + String(html.previousElementSibling) + ':' + String(head.nextElementSibling) + ':' + String(head.previousElementSibling) + ':' + String(body.nextElementSibling) + ':' + String(body.previousElementSibling) + ':' + String(text.nextElementSibling) + ':' + String(text.previousElementSibling) + ':' + String(out.nextElementSibling) + ':' + String(out.previousElementSibling) + ':' + String(main.previousElementSibling) + ':' + String(main.nextElementSibling) + ':' + String(script.previousElementSibling) + ':' + String(script.nextElementSibling);",
            "inline-script",
            &mut host,
        )
        .expect("element sibling reflection should resolve through the script runtime");

    assert_eq!(
        host.text_content
            .get(&ElementHandle::new(5))
            .map(String::as_str),
        Some(
            "null:null:null:null:[object Element]:null:null:[object Element]:[object Element]:null:[object Element]:null:[object Element]:[object Element]:[object Element]:null"
        )
    );
}

#[test]
fn runtime_rejects_sibling_assignment() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("body", ElementHandle::new(3), "");
    host.seed_document_body(Some(ElementHandle::new(3)));
    host.seed_node_child_nodes_items(
        HtmlCollectionScope::Element(ElementHandle::new(3)),
        vec![NodeHandle::new(4), NodeHandle::new(5)],
    );
    host.seed_node_parent(NodeHandle::new(4), Some(NodeHandle::new(3)));
    host.seed_node_parent(NodeHandle::new(5), Some(NodeHandle::new(3)));
    host.seed_node_type(NodeHandle::new(3), 1);
    host.seed_node_type(NodeHandle::new(4), 3);
    host.seed_node_type(NodeHandle::new(5), 1);

    let error = runtime
        .eval_program(
            "document.body.nextSibling = null;",
            "inline-script",
            &mut host,
        )
        .expect_err("nextSibling should be read-only");

    assert!(error.to_string().contains("unsupported assignment target"));
    assert!(error.to_string().contains("nextSibling"));
}

#[test]
fn runtime_rejects_element_sibling_assignment() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("body", ElementHandle::new(3), "");
    host.seed_document_body(Some(ElementHandle::new(3)));
    host.seed_node_child_nodes_items(
        HtmlCollectionScope::Element(ElementHandle::new(3)),
        vec![NodeHandle::new(4), NodeHandle::new(5), NodeHandle::new(6)],
    );
    host.seed_node_parent(NodeHandle::new(4), Some(NodeHandle::new(3)));
    host.seed_node_parent(NodeHandle::new(5), Some(NodeHandle::new(3)));
    host.seed_node_parent(NodeHandle::new(6), Some(NodeHandle::new(3)));
    host.seed_node_type(NodeHandle::new(3), 1);
    host.seed_node_type(NodeHandle::new(4), 3);
    host.seed_node_type(NodeHandle::new(5), 1);
    host.seed_node_type(NodeHandle::new(6), 1);

    let error = runtime
        .eval_program(
            "document.body.nextElementSibling = null;",
            "inline-script",
            &mut host,
        )
        .expect_err("nextElementSibling should be read-only");

    assert!(error.to_string().contains("unsupported assignment target"));
    assert!(error.to_string().contains("on element"));
    assert!(error.to_string().contains("nextElementSibling"));
}

#[test]
fn runtime_rejects_has_child_nodes_wrong_arity() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program("document.hasChildNodes(true);", "inline-script", &mut host)
        .expect_err("hasChildNodes should validate arity");

    assert!(
        error
            .to_string()
            .contains("hasChildNodes() expects no arguments")
    );
}

#[test]
fn runtime_rejects_contains_wrong_arity() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();

    let error = runtime
        .eval_program("document.contains();", "inline-script", &mut host)
        .expect_err("contains should validate arity");

    assert!(
        error
            .to_string()
            .contains("contains() expects exactly one argument")
    );
}
