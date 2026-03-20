use std::collections::BTreeMap;

use bt_script::{
    ElementHandle, HostBindings, HtmlCollectionScope, HtmlCollectionTarget, ListenerTarget,
    ScriptFunction, ScriptRuntime,
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

#[derive(Default)]
struct RecordingHost {
    elements: BTreeMap<String, ElementHandle>,
    text_content: BTreeMap<ElementHandle, String>,
    values: BTreeMap<ElementHandle, String>,
    checked: BTreeMap<ElementHandle, bool>,
    attributes: BTreeMap<(ElementHandle, String), String>,
    element_children_results: BTreeMap<ElementHandle, Vec<ElementHandle>>,
    html_collection_tag_name_items_results: BTreeMap<HtmlCollectionTarget, Vec<ElementHandle>>,
    html_collection_tag_name_ns_items_results: BTreeMap<HtmlCollectionTarget, Vec<ElementHandle>>,
    html_collection_class_name_items_results: BTreeMap<HtmlCollectionTarget, Vec<ElementHandle>>,
    html_collection_form_elements_items_results: BTreeMap<ElementHandle, Vec<ElementHandle>>,
    html_collection_select_options_items_results: BTreeMap<ElementHandle, Vec<ElementHandle>>,
    document_links_items_results: Vec<ElementHandle>,
    document_anchors_items_results: Vec<ElementHandle>,
    document_children_items_results: Vec<ElementHandle>,
    html_collection_named_item_results: BTreeMap<(ElementHandle, String), Option<ElementHandle>>,
    html_collection_tag_name_named_item_results:
        BTreeMap<(HtmlCollectionTarget, String), Option<ElementHandle>>,
    html_collection_tag_name_ns_named_item_results:
        BTreeMap<(HtmlCollectionTarget, String), Option<ElementHandle>>,
    html_collection_class_name_named_item_results:
        BTreeMap<(HtmlCollectionTarget, String), Option<ElementHandle>>,
    html_collection_form_elements_named_item_results:
        BTreeMap<(ElementHandle, String), Option<ElementHandle>>,
    html_collection_select_options_named_item_results:
        BTreeMap<(ElementHandle, String), Option<ElementHandle>>,
    document_links_named_item_results: BTreeMap<String, Option<ElementHandle>>,
    document_anchors_named_item_results: BTreeMap<String, Option<ElementHandle>>,
    document_children_named_item_results: BTreeMap<String, Option<ElementHandle>>,
    document_query_selector_results: BTreeMap<String, Option<ElementHandle>>,
    document_query_selector_all_results: BTreeMap<String, Vec<ElementHandle>>,
    document_get_elements_by_name_results: BTreeMap<String, Vec<ElementHandle>>,
    element_query_selector_results: BTreeMap<(ElementHandle, String), Option<ElementHandle>>,
    element_query_selector_all_results: BTreeMap<(ElementHandle, String), Vec<ElementHandle>>,
    element_closest_results: BTreeMap<(ElementHandle, String), Option<ElementHandle>>,
    element_children_calls: Vec<ElementHandle>,
    html_collection_tag_name_items_calls: Vec<HtmlCollectionTarget>,
    html_collection_tag_name_ns_items_calls: Vec<HtmlCollectionTarget>,
    html_collection_class_name_items_calls: Vec<HtmlCollectionTarget>,
    html_collection_form_elements_items_calls: Vec<ElementHandle>,
    html_collection_select_options_items_calls: Vec<ElementHandle>,
    document_links_items_calls: usize,
    document_anchors_items_calls: usize,
    document_children_items_calls: usize,
    html_collection_named_item_calls: Vec<(ElementHandle, String)>,
    html_collection_tag_name_named_item_calls: Vec<(HtmlCollectionTarget, String)>,
    html_collection_tag_name_ns_named_item_calls: Vec<(HtmlCollectionTarget, String)>,
    html_collection_class_name_named_item_calls: Vec<(HtmlCollectionTarget, String)>,
    html_collection_form_elements_named_item_calls: Vec<(ElementHandle, String)>,
    html_collection_select_options_named_item_calls: Vec<(ElementHandle, String)>,
    document_links_named_item_calls: Vec<String>,
    document_anchors_named_item_calls: Vec<String>,
    document_children_named_item_calls: Vec<String>,
    document_query_selector_calls: Vec<String>,
    document_query_selector_all_calls: Vec<String>,
    document_get_elements_by_name_calls: Vec<String>,
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

    fn seed_document_links_items(&mut self, result: Vec<ElementHandle>) {
        self.document_links_items_results = result;
    }

    fn seed_document_anchors_items(&mut self, result: Vec<ElementHandle>) {
        self.document_anchors_items_results = result;
    }

    fn seed_document_children_items(&mut self, result: Vec<ElementHandle>) {
        self.document_children_items_results = result;
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

    fn seed_html_collection_select_options_named_item(
        &mut self,
        element: ElementHandle,
        name: impl Into<String>,
        result: Option<ElementHandle>,
    ) {
        self.html_collection_select_options_named_item_results
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

    fn seed_document_query_selector(
        &mut self,
        selector: impl Into<String>,
        result: Option<ElementHandle>,
    ) {
        self.document_query_selector_results
            .insert(selector.into(), result);
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

    fn element_set_text_content(
        &mut self,
        element: ElementHandle,
        value: &str,
    ) -> bt_script::Result<()> {
        self.text_content.insert(element, value.to_string());
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
