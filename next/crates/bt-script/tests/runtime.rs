use std::collections::BTreeMap;

use bt_script::{ElementHandle, HostBindings, ListenerTarget, ScriptFunction, ScriptRuntime};

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
    document_query_selector_results: BTreeMap<String, Option<ElementHandle>>,
    element_query_selector_results: BTreeMap<(ElementHandle, String), Option<ElementHandle>>,
    element_closest_results: BTreeMap<(ElementHandle, String), Option<ElementHandle>>,
    document_query_selector_calls: Vec<String>,
    element_query_selector_calls: Vec<(ElementHandle, String)>,
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

    fn seed_document_query_selector(
        &mut self,
        selector: impl Into<String>,
        result: Option<ElementHandle>,
    ) {
        self.document_query_selector_results
            .insert(selector.into(), result);
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
            "document.querySelectorAll('#out').textContent = 'Hello';",
            "inline-script",
            &mut host,
        )
        .expect_err("querySelectorAll should not be supported yet");

    assert!(
        error
            .to_string()
            .contains("unsupported Document method: querySelectorAll")
    );
}
