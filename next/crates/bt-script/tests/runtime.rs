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
    element_children_results: BTreeMap<ElementHandle, Vec<ElementHandle>>,
    html_collection_tag_name_items_results: BTreeMap<HtmlCollectionTarget, Vec<ElementHandle>>,
    html_collection_class_name_items_results: BTreeMap<HtmlCollectionTarget, Vec<ElementHandle>>,
    html_collection_named_item_results: BTreeMap<(ElementHandle, String), Option<ElementHandle>>,
    html_collection_tag_name_named_item_results:
        BTreeMap<(HtmlCollectionTarget, String), Option<ElementHandle>>,
    html_collection_class_name_named_item_results:
        BTreeMap<(HtmlCollectionTarget, String), Option<ElementHandle>>,
    document_query_selector_results: BTreeMap<String, Option<ElementHandle>>,
    document_query_selector_all_results: BTreeMap<String, Vec<ElementHandle>>,
    document_get_elements_by_name_results: BTreeMap<String, Vec<ElementHandle>>,
    element_query_selector_results: BTreeMap<(ElementHandle, String), Option<ElementHandle>>,
    element_query_selector_all_results: BTreeMap<(ElementHandle, String), Vec<ElementHandle>>,
    element_closest_results: BTreeMap<(ElementHandle, String), Option<ElementHandle>>,
    element_children_calls: Vec<ElementHandle>,
    html_collection_tag_name_items_calls: Vec<HtmlCollectionTarget>,
    html_collection_class_name_items_calls: Vec<HtmlCollectionTarget>,
    html_collection_named_item_calls: Vec<(ElementHandle, String)>,
    html_collection_tag_name_named_item_calls: Vec<(HtmlCollectionTarget, String)>,
    html_collection_class_name_named_item_calls: Vec<(HtmlCollectionTarget, String)>,
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

    fn seed_html_collection_class_name_items(
        &mut self,
        collection: HtmlCollectionTarget,
        result: Vec<ElementHandle>,
    ) {
        self.html_collection_class_name_items_results
            .insert(collection, result);
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

    fn seed_html_collection_class_name_named_item(
        &mut self,
        collection: HtmlCollectionTarget,
        name: impl Into<String>,
        result: Option<ElementHandle>,
    ) {
        self.html_collection_class_name_named_item_results
            .insert((collection, name.into()), result);
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

    fn document_get_elements_by_name(&mut self, name: &str) -> bt_script::Result<Vec<ElementHandle>> {
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

    assert!(error.to_string().contains("unsupported Element method: getElementsByName"));
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
fn runtime_reports_unsupported_syntax_explicitly() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("out", ElementHandle::new(1), "");

    let error = runtime
        .eval_program(
            "document.querySelectorAll('#out').forEach(() => {});",
            "inline-script",
            &mut host,
        )
        .expect_err("unsupported node list methods should fail");

    assert!(
        error
            .to_string()
            .contains("unsupported NodeList method: forEach")
    );
}

#[test]
fn runtime_reports_html_collection_methods_explicitly() {
    let mut runtime = ScriptRuntime::new();
    let mut host = RecordingHost::default();
    host.seed_element("root", ElementHandle::new(1), "");
    host.seed_element("out", ElementHandle::new(2), "");
    host.seed_element_children(ElementHandle::new(1), vec![ElementHandle::new(3)]);

    let error = runtime
        .eval_program(
            "document.getElementById('root').children.forEach(() => {});",
            "inline-script",
            &mut host,
        )
        .expect_err("unsupported html collection methods should fail");

    assert!(
        error
            .to_string()
            .contains("unsupported HTMLCollection method: forEach")
    );
}
