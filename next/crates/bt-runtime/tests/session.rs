use std::collections::BTreeMap;

use bt_runtime::{MatchMediaListenerCall, ScrollMethod, Session, SessionConfig};
use bt_script::ScriptRuntime;

#[test]
fn session_keeps_builder_configuration() {
    let mut local_storage = BTreeMap::new();
    local_storage.insert("theme".to_string(), "light".to_string());

    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some("<div id='app'></div>".to_string()),
        local_storage,
    })
    .expect("session should parse HTML");

    assert_eq!(session.config().url, "https://example.test/app");
    assert_eq!(session.dom().source_html(), Some("<div id='app'></div>"));
    assert_eq!(session.dom().node_count(), 2);
    assert_eq!(
        session
            .mocks()
            .storage()
            .local()
            .get("theme")
            .map(String::as_str),
        Some("light")
    );
    assert!(session.mocks().storage().session().is_empty());
    assert_eq!(session.scheduler().now_ms(), 0);
    assert!(!session.debug().trace_enabled());
}

#[test]
fn session_starts_with_empty_storage_seed_registry() {
    let session = Session::new(SessionConfig::default()).expect("session should build");

    assert_eq!(session.config().url, "https://app.local/");
    assert!(session.mocks().storage().local().is_empty());
    assert!(session.mocks().storage().session().is_empty());
    assert_eq!(session.dom().node_count(), 1);
}

#[test]
fn session_rejects_malformed_html() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some("<div><span></div>".to_string()),
        local_storage: BTreeMap::new(),
    })
    .expect_err("malformed HTML should fail");

    assert!(error.to_string().contains("mismatched closing tag"));
}

#[test]
fn session_executes_inline_scripts_during_bootstrap() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>document.getElementById('out').textContent = 'Hello';</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute inline scripts");

    assert_eq!(
        session.dom().dump_dom(),
        "#document\n  <main id=\"out\">\n    \"Hello\"\n  </main>\n  <script>\n    \"document.getElementById('out').textContent = 'Hello';\"\n  </script>"
    );
}

#[test]
fn session_resolves_document_root_head_and_body() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<html id='html'><head id='head'><title>Title</title></head><body id='body'><main id='out'></main><script>const html = document.documentElement; const head = document.head; const body = document.body; const scrolling = document.scrollingElement; document.getElementById('out').textContent = html.getAttribute('id') + ':' + head.getAttribute('id') + ':' + body.getAttribute('id') + ':' + scrolling.getAttribute('id');</script></body></html>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose document root/head/body");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "html:head:body:html"
    );
}

#[test]
fn session_resolves_owner_document_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<html><body id='body'><main id='out'></main><script>document.getElementById('out').textContent = String(document.body.ownerDocument) + ':' + String(document.body.ownerDocument.defaultView) + ':' + String(document.ownerDocument);</script></body></html>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose ownerDocument");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "[object Document]:[object Window]:null"
    );
}

#[test]
fn session_resolves_parent_node_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<html><body id='body'>Text<main id='out'></main><script>const body = document.body; const text = body.childNodes.item(0); document.getElementById('out').textContent = String(document.parentNode) + ':' + String(document.documentElement.parentNode) + ':' + String(document.documentElement.parentElement) + ':' + String(body.parentNode) + ':' + String(body.parentElement) + ':' + String(text.parentNode) + ':' + String(text.parentElement);</script></body></html>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose parentNode");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "null:[object Document]:null:[object Element]:[object Element]:[object Element]:[object Element]"
    );
}

#[test]
fn session_resolves_element_tag_name_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'></main><div id='out'></div><script>document.getElementById('out').textContent = document.getElementById('root').tagName;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose element.tagName");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "main");
}

#[test]
fn session_resolves_element_local_name_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'></main><div id='out'></div><script>document.getElementById('out').textContent = document.getElementById('root').localName;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose element.localName");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "main");
}

#[test]
fn session_resolves_element_namespace_uri_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'></main><div id='out'></div><script>document.getElementById('out').textContent = document.getElementById('root').namespaceURI;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose element.namespaceURI");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "http://www.w3.org/1999/xhtml"
    );
}

#[test]
fn session_resolves_create_element_ns_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some("<main id='root'></main><div id='out'></div><script>const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg'); svg.setAttribute('viewBox', '0 0 10 10'); document.getElementById('root').appendChild(svg); document.getElementById('out').textContent = svg.namespaceURI + '|' + svg.localName + '|' + svg.outerHTML;</script>".to_string()),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose document.createElementNS");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "http://www.w3.org/2000/svg|svg|<svg viewBox=\"0 0 10 10\"></svg>"
    );
}

#[test]
fn session_rejects_create_element_ns_invalid_tag_name_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'></main><script>document.createElementNS('http://www.w3.org/2000/svg', 'svg:rect');</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("invalid qualified names should fail explicitly");

    let message = error.to_string();
    assert!(message.contains("Script error"));
    assert!(message.contains("invalid tag name"));
    assert!(message.contains("svg:rect"));
}

#[test]
fn session_resolves_element_name_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root' name='root'></main><div id='out'></div><script>const root = document.getElementById('root'); const before = document.getElementsByName('root').length; root.name = 'next'; const next = document.getElementsByName('next'); document.getElementById('out').textContent = String(before) + ':' + String(document.getElementsByName('root').length) + ':' + String(next.length) + ':' + next.item(0).getAttribute('id');</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose element.name");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "1:0:1:root");
}

#[test]
fn session_rejects_parent_node_assignment_through_inline_scripts() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<body id='body'><script>document.body.parentNode = null;</script></body>".to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("parentNode should be read-only");

    assert!(error.to_string().contains("unsupported assignment target"));
    assert!(error.to_string().contains("parentNode"));
}

#[test]
fn session_resolves_compare_document_position_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<html><body><main id='root'><span id='a'><em id='c'>Child</em></span><span id='b'>Sibling</span></main><div id='out'></div><script>const a = document.getElementById('a'); const b = document.getElementById('b'); const c = document.getElementById('c'); const detached = document.createElement('p'); document.getElementById('out').textContent = String(document.compareDocumentPosition(a)) + ':' + String(a.compareDocumentPosition(document)) + ':' + String(a.compareDocumentPosition(b)) + ':' + String(b.compareDocumentPosition(a)) + ':' + String(a.compareDocumentPosition(c)) + ':' + String(c.compareDocumentPosition(a)) + ':' + String(a.compareDocumentPosition(detached)) + ':' + String(detached.compareDocumentPosition(a)) + ':' + String(document.compareDocumentPosition(document));</script></body></html>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose compareDocumentPosition");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "20:10:4:2:20:10:37:35:0"
    );
}

#[test]
fn session_resolves_is_same_node_and_is_equal_node_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<html><body><main id='root'><span id='child'>Child</span></main><template id='tpl'><span id='inner'>Inner</span></template><div id='out'></div><script>const root = document.getElementById('root'); const tpl = document.getElementById('tpl'); const clone = root.cloneNode(true); const fragment = tpl.content.cloneNode(true); document.getElementById('out').textContent = String(document.isSameNode(document)) + ':' + String(document.isSameNode(null)) + ':' + String(root.isSameNode(clone)) + ':' + String(root.isEqualNode(clone)) + ':' + String(tpl.content.isSameNode(fragment)) + ':' + String(tpl.content.isEqualNode(fragment));</script></body></html>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose isSameNode and isEqualNode");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "true:false:false:true:false:true"
    );
}

#[test]
fn session_resolves_is_connected_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<html><body><div id='ghost'></div><main id='out'></main><script>const ghost = document.getElementById('ghost'); ghost.remove(); document.getElementById('out').textContent = String(document.isConnected) + ':' + String(document.body.isConnected) + ':' + String(ghost.isConnected);</script></body></html>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose isConnected");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "true:true:false"
    );
}

#[test]
fn session_rejects_is_connected_assignment_through_inline_scripts() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<body id='body'><script>document.body.isConnected = false;</script></body>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("isConnected should be read-only");

    assert!(error.to_string().contains("unsupported assignment target"));
    assert!(error.to_string().contains("isConnected"));
}

#[test]
fn session_resolves_first_and_last_element_child_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<html><head></head><body><div id='wrapper'><span></span><span></span></div><template id='tmpl'><span></span><span></span></template><main id='out'></main><script>const html = document.documentElement; const wrapper = document.getElementById('wrapper'); const template = document.getElementById('tmpl').content; document.getElementById('out').textContent = String(document.childElementCount) + ':' + String(document.firstElementChild) + ':' + String(document.lastElementChild) + ':' + String(html.childElementCount) + ':' + String(html.firstElementChild) + ':' + String(html.lastElementChild) + ':' + String(wrapper.childElementCount) + ':' + String(wrapper.firstElementChild) + ':' + String(wrapper.lastElementChild) + ':' + String(template.childElementCount) + ':' + String(template.firstElementChild) + ':' + String(template.lastElementChild);</script></body></html>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose first/lastElementChild");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "1:[object Element]:[object Element]:2:[object Element]:[object Element]:2:[object Element]:[object Element]:2:[object Element]:[object Element]"
    );
}

#[test]
fn session_rejects_first_element_child_assignment_through_inline_scripts() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<body id='body'><script>document.body.firstElementChild = null;</script></body>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("firstElementChild should be read-only");

    assert!(error.to_string().contains("unsupported assignment target"));
    assert!(error.to_string().contains("firstElementChild"));
}

#[test]
fn session_resolves_document_title_getter_setter_and_window_alias() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<html><head><title>Initial</title></head><body><main id='out'></main><script>const before = document.title; document.title = 'Updated'; const after = window.title; document.getElementById('out').textContent = before + ':' + after + ':' + document.querySelector('title').textContent;</script></body></html>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose document.title");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "Initial:Updated:Updated"
    );
    assert_eq!(session.dom().document_title(), "Updated");
}

#[test]
fn session_resolves_element_title_getter_setter_and_attribute_reflection() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><div id='box' title='Initial'></div><div id='out'></div><script>const box = document.getElementById('box'); const before = box.title; box.title = 'Updated'; document.getElementById('out').textContent = before + ':' + box.title + ':' + box.getAttribute('title');</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose element.title");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "Initial:Updated:Updated"
    );
}

#[test]
fn session_resolves_element_role_getter_setter_and_attribute_reflection() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><div id='box' role='button'></div><div id='out'></div><script>const box = document.getElementById('box'); const before = box.role; box.role = 'menu'; document.getElementById('out').textContent = before + ':' + box.role + ':' + box.getAttribute('role');</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose element.role");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "button:menu:menu"
    );
}

#[test]
fn session_resolves_element_tab_index_getter_setter_and_attribute_reflection() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><div id='box'></div><button id='button'></button><div id='out'></div><script>const box = document.getElementById('box'); const button = document.getElementById('button'); const before = String(box.tabIndex) + '|' + String(button.tabIndex); box.tabIndex = 4; button.tabIndex = -1; document.getElementById('out').textContent = before + ':' + String(box.tabIndex) + '|' + String(button.tabIndex) + ':' + box.getAttribute('tabindex') + '|' + button.getAttribute('tabindex');</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose element.tabIndex");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "-1|0:4|-1:4|-1"
    );
}

#[test]
fn session_resolves_element_aria_label_getter_setter_and_attribute_reflection() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><div id='box' aria-label='Initial'></div><div id='out'></div><script>const box = document.getElementById('box'); const before = box.ariaLabel; box.ariaLabel = 'Updated'; document.getElementById('out').textContent = before + ':' + box.ariaLabel + ':' + box.getAttribute('aria-label');</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose element.ariaLabel");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "Initial:Updated:Updated"
    );
}

#[test]
fn session_resolves_element_aria_description_getter_setter_and_attribute_reflection() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><div id='box' aria-description='Initial'></div><div id='out'></div><script>const box = document.getElementById('box'); const before = box.ariaDescription; box.ariaDescription = 'Updated'; document.getElementById('out').textContent = before + ':' + box.ariaDescription + ':' + box.getAttribute('aria-description');</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose element.ariaDescription");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "Initial:Updated:Updated"
    );
}

#[test]
fn session_resolves_element_aria_role_description_getter_setter_and_attribute_reflection() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><div id='box' aria-roledescription='Initial'></div><div id='out'></div><script>const box = document.getElementById('box'); const before = box.ariaRoleDescription; box.ariaRoleDescription = 'Updated'; document.getElementById('out').textContent = before + ':' + box.ariaRoleDescription + ':' + box.getAttribute('aria-roledescription');</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose element.ariaRoleDescription");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "Initial:Updated:Updated"
    );
}

#[test]
fn session_resolves_element_aria_hidden_getter_setter_and_attribute_reflection() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><div id='box' aria-hidden='true'></div><div id='out'></div><script>const box = document.getElementById('box'); const before = box.ariaHidden; box.ariaHidden = false; document.getElementById('out').textContent = before + ':' + box.ariaHidden + ':' + box.getAttribute('aria-hidden');</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose element.ariaHidden");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "true:false:false"
    );
}

#[test]
fn session_resolves_element_access_key_getter_setter_and_attribute_reflection() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><div id='box' accesskey='x'></div><div id='out'></div><script>const box = document.getElementById('box'); const before = box.accessKey; box.accessKey = 'y'; document.getElementById('out').textContent = before + ':' + box.accessKey + ':' + box.getAttribute('accesskey');</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose element.accessKey");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "x:y:y");
}

#[test]
fn session_resolves_element_slot_getter_setter_and_attribute_reflection() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><div id='box' slot='start'></div><div id='out'></div><script>const box = document.getElementById('box'); const before = box.slot; box.slot = 'end'; document.getElementById('out').textContent = before + ':' + box.slot + ':' + box.getAttribute('slot');</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose element.slot");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "start:end:end");
}

#[test]
fn session_resolves_element_autocapitalize_getter_setter_and_attribute_reflection() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><div id='box' autocapitalize='sentences'></div><div id='out'></div><script>const box = document.getElementById('box'); const before = box.autocapitalize; box.autocapitalize = 'words'; document.getElementById('out').textContent = before + ':' + box.autocapitalize + ':' + box.getAttribute('autocapitalize');</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose element.autocapitalize");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "sentences:words:words"
    );
}

#[test]
fn session_resolves_element_hidden_getter_setter_and_attribute_reflection() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><div id='box' hidden></div><div id='out'></div><script>const box = document.getElementById('box'); const before = String(box.hidden); box.hidden = false; const afterClear = String(box.hidden) + ':' + String(box.hasAttribute('hidden')); box.hidden = true; document.getElementById('out').textContent = before + ':' + afterClear + ':' + String(box.hidden) + ':' + String(box.hasAttribute('hidden'));</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose element.hidden");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "true:false:false:true:true"
    );
}

#[test]
fn session_resolves_document_location_getter_setter_and_window_alias() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/start".to_string(),
        html: Some(
            "<main id='out'></main><script>const before = document.location; document.location = 'https://example.test/next'; const after = window.location; document.getElementById('out').textContent = before + ':' + after;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose document.location");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "https://example.test/start:https://example.test/next"
    );
    assert_eq!(
        session.mocks().location().current_url(),
        Some("https://example.test/next")
    );
    assert_eq!(
        session.mocks().location().navigations(),
        &["https://example.test/next".to_string()]
    );
    assert_eq!(session.document_location(), "https://example.test/next");
}

#[test]
fn session_resolves_location_assign_replace_and_reload_methods() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>const before = document.location; window.location.assign('https://example.test/assign'); document.location.replace('https://example.test/replace'); window.location.reload(); document.getElementById('out').textContent = before + ':' + document.location + ':' + String(window.history.length);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose location assign/replace/reload");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "https://example.test/app:https://example.test/replace:2"
    );
    assert_eq!(
        session.mocks().location().current_url(),
        Some("https://example.test/replace")
    );
    assert_eq!(
        session.mocks().location().navigations(),
        &[
            "https://example.test/assign".to_string(),
            "https://example.test/replace".to_string(),
            "https://example.test/replace".to_string(),
        ]
    );
    assert_eq!(session.document_location(), "https://example.test/replace");
}

#[test]
fn session_resolves_location_href_getter_and_setter() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>const before = window.location.href; document.location.href = 'https://example.test/next'; const after = document.location.href; document.getElementById('out').textContent = before + ':' + after;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose location.href");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "https://example.test/app:https://example.test/next"
    );
    assert_eq!(
        session.mocks().location().current_url(),
        Some("https://example.test/next")
    );
    assert_eq!(
        session.mocks().location().navigations(),
        &["https://example.test/next".to_string()]
    );
    assert_eq!(session.document_location(), "https://example.test/next");
}

#[test]
fn session_resolves_location_stringification_helpers() {
    let session = Session::new(SessionConfig {
        url: "https://example.test:8443/start?x#old".to_string(),
        html: Some(
            "<main id='out'></main><script>document.getElementById('out').textContent = document.location.toString() + ':' + window.location.valueOf();</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose location stringification helpers");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "https://example.test:8443/start?x#old:https://example.test:8443/start?x#old"
    );
    assert_eq!(
        session.mocks().location().current_url(),
        Some("https://example.test:8443/start?x#old")
    );
}

#[test]
fn session_resolves_location_hash_getter_and_setter() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>const before = window.location.hash; document.location.hash = '#next'; const after = document.location.hash; document.getElementById('out').textContent = before + ':' + document.location + ':' + after;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose location.hash");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        ":https://example.test/app#next:#next"
    );
    assert_eq!(
        session.mocks().location().current_url(),
        Some("https://example.test/app#next")
    );
    assert_eq!(
        session.mocks().location().navigations(),
        &["https://example.test/app#next".to_string()]
    );
    assert_eq!(session.document_location(), "https://example.test/app#next");
}

#[test]
fn session_resolves_location_pathname_getter_and_setter() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/start?x#old".to_string(),
        html: Some(
            "<main id='out'></main><script>const before = window.location.pathname; document.location.pathname = 'next'; const after = document.location.pathname; document.getElementById('out').textContent = before + ':' + document.location + ':' + after;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose location.pathname");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "/start:https://example.test/next?x#old:/next"
    );
    assert_eq!(
        session.mocks().location().current_url(),
        Some("https://example.test/next?x#old")
    );
    assert_eq!(
        session.mocks().location().navigations(),
        &["https://example.test/next?x#old".to_string()]
    );
    assert_eq!(
        session.document_location(),
        "https://example.test/next?x#old"
    );
}

#[test]
fn session_resolves_location_search_getter_and_setter() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/start?x#old".to_string(),
        html: Some(
            "<main id='out'></main><script>const before = window.location.search; document.location.search = '?next'; const after = document.location.search; document.getElementById('out').textContent = before + ':' + document.location + ':' + after;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose location.search");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "?x:https://example.test/start?next#old:?next"
    );
    assert_eq!(
        session.mocks().location().current_url(),
        Some("https://example.test/start?next#old")
    );
    assert_eq!(
        session.mocks().location().navigations(),
        &["https://example.test/start?next#old".to_string()]
    );
    assert_eq!(
        session.document_location(),
        "https://example.test/start?next#old"
    );
}

#[test]
fn session_resolves_location_origin_getter() {
    let session = Session::new(SessionConfig {
        url: "https://example.test:8443/start?x#old".to_string(),
        html: Some(
            "<main id='out'></main><script>const before = window.location.origin; document.location.pathname = 'next'; const after = document.location.origin; document.getElementById('out').textContent = before + ':' + after + ':' + document.location;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose location.origin");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "https://example.test:8443:https://example.test:8443:https://example.test:8443/next?x#old"
    );
    assert_eq!(
        session.mocks().location().current_url(),
        Some("https://example.test:8443/next?x#old")
    );
    assert_eq!(
        session.mocks().location().navigations(),
        &["https://example.test:8443/next?x#old".to_string()]
    );
    assert_eq!(
        session.document_location(),
        "https://example.test:8443/next?x#old"
    );
}

#[test]
fn session_resolves_location_protocol_host_hostname_and_port_getters_and_setters() {
    let session = Session::new(SessionConfig {
        url: "https://app.local:8443/start?x#old".to_string(),
        html: Some(
            "<main id='out'></main><script>const before = window.location.protocol + '|' + window.location.host + '|' + window.location.hostname + '|' + window.location.port; document.location.protocol = 'http:'; document.location.host = 'example.test:8080'; document.location.hostname = 'example.test'; document.location.port = '8080'; const after = window.location.protocol + '|' + window.location.host + '|' + window.location.hostname + '|' + window.location.port; document.getElementById('out').textContent = before + ':' + after + ':' + document.location;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose location protocol/host/hostname/port");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "https:|app.local:8443|app.local|8443:http:|example.test:8080|example.test|8080:http://example.test:8080/start?x#old"
    );
    assert_eq!(
        session.mocks().location().current_url(),
        Some("http://example.test:8080/start?x#old")
    );
    assert_eq!(
        session.mocks().location().navigations(),
        &[
            "http://app.local:8443/start?x#old".to_string(),
            "http://example.test:8080/start?x#old".to_string(),
            "http://example.test:8080/start?x#old".to_string(),
            "http://example.test:8080/start?x#old".to_string(),
        ]
    );
    assert_eq!(
        session.document_location(),
        "http://example.test:8080/start?x#old"
    );
}

#[test]
fn session_resolves_location_username_and_password_getters_and_setters() {
    let session = Session::new(SessionConfig {
        url: "https://alice:secret@example.test:8443/start?x#old".to_string(),
        html: Some(
            "<main id='out'></main><script>const before = window.location.username + '|' + window.location.password; document.location.username = 'bob'; document.location.password = 'hunter2'; document.location.port = '9444'; const after = window.location.username + '|' + window.location.password + '|' + window.location.port; document.getElementById('out').textContent = before + ':' + after + ':' + document.location;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose location username/password");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "alice|secret:bob|hunter2|9444:https://bob:hunter2@example.test:9444/start?x#old"
    );
    assert_eq!(
        session.mocks().location().current_url(),
        Some("https://bob:hunter2@example.test:9444/start?x#old")
    );
    assert_eq!(
        session.mocks().location().navigations(),
        &[
            "https://bob:secret@example.test:8443/start?x#old".to_string(),
            "https://bob:hunter2@example.test:8443/start?x#old".to_string(),
            "https://bob:hunter2@example.test:9444/start?x#old".to_string(),
        ]
    );
    assert_eq!(
        session.document_location(),
        "https://bob:hunter2@example.test:9444/start?x#old"
    );
}

#[test]
fn session_rejects_location_port_with_non_numeric_value() {
    let error = Session::new(SessionConfig {
        url: "https://example.test:8443/start?x#old".to_string(),
        html: Some("<script>document.location.port = 'abc';</script>".to_string()),
        local_storage: BTreeMap::new(),
    })
    .expect_err("session should reject non-numeric location.port assignments");

    assert!(
        error
            .to_string()
            .contains("unsupported location.port value: abc")
    );
}

#[test]
fn session_rejects_location_assign_without_url() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some("<script>window.location.assign();</script>".to_string()),
        local_storage: BTreeMap::new(),
    })
    .expect_err("session should reject location.assign without a URL");

    assert!(
        error
            .to_string()
            .contains("location.assign() expects exactly one argument")
    );
}

#[test]
fn session_rejects_location_href_without_url() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some("<script>window.location.href = '';</script>".to_string()),
        local_storage: BTreeMap::new(),
    })
    .expect_err("session should reject location.href without a URL");

    assert!(
        error
            .to_string()
            .contains("navigate() requires a non-empty URL")
    );
}

#[test]
fn session_resolves_document_cookie_getter_and_setter() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/start".to_string(),
        html: Some(
            "<main id='out'></main><script>document.cookie = 'theme=dark'; document.cookie = 'theme=light'; document.getElementById('out').textContent = document.cookie;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose document.cookie");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "theme=light");
}

#[test]
fn session_rejects_malformed_document_cookie_assignment() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/start".to_string(),
        html: Some("<script>document.cookie = 'badcookie';</script>".to_string()),
        local_storage: BTreeMap::new(),
    })
    .expect_err("malformed cookie assignments should fail explicitly");

    assert!(
        error
            .to_string()
            .contains("document.cookie requires `name=value`")
    );
}

#[test]
fn session_resolves_document_url_and_document_uri_aliases() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/start".to_string(),
        html: Some(
            "<main id='out'></main><script>const beforeLocation = document.location; const beforeUrl = document.URL; const beforeDocumentUri = document.documentURI; const beforeWindowLocation = window.location; document.getElementById('out').textContent = beforeLocation + ':' + beforeUrl + ':' + beforeDocumentUri + ':' + beforeWindowLocation;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose document.URL aliases");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "https://example.test/start:https://example.test/start:https://example.test/start:https://example.test/start"
    );
    assert_eq!(session.document_location(), "https://example.test/start");
}

#[test]
fn session_resolves_document_base_uri_and_element_base_uri_aliases() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/start".to_string(),
        html: Some(
            "<main id='root'><span id='child'></span></main><div id='out'></div><script>const root = document.getElementById('root'); const child = document.getElementById('child'); document.getElementById('out').textContent = document.baseURI + ':' + root.baseURI + ':' + child.baseURI;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose baseURI aliases");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "https://example.test/start:https://example.test/start:https://example.test/start"
    );
    assert_eq!(session.document_base_uri(), "https://example.test/start");
}

#[test]
fn session_resolves_document_origin_and_element_origin_aliases() {
    let session = Session::new(SessionConfig {
        url: "https://example.test:8443/start?x#y".to_string(),
        html: Some(
            "<main id='root'><span id='child'></span></main><div id='out'></div><script>const root = document.getElementById('root'); const child = document.getElementById('child'); document.getElementById('out').textContent = document.domain + ':' + document.origin + ':' + window.origin + ':' + root.origin + ':' + child.origin;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose domain and origin aliases");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "example.test:https://example.test:8443:https://example.test:8443:https://example.test:8443:https://example.test:8443"
    );
    assert_eq!(session.document_domain(), "example.test");
    assert_eq!(session.document_origin(), "https://example.test:8443");
}

#[test]
fn session_rejects_document_domain_assignment() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some("<script>document.domain = 'example.test';</script>".to_string()),
        local_storage: BTreeMap::new(),
    })
    .expect_err("document.domain should be read-only");

    assert!(error.to_string().contains("unsupported assignment target"));
    assert!(error.to_string().contains("domain"));
}

#[test]
fn session_resolves_web_storage_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>const local = window.localStorage; const session = document.defaultView.sessionStorage; const before = String(local) + ':' + String(session) + ':' + String(local.length) + ':' + String(session.length); const token = local.token; local.theme = 'dark'; local.removeItem('token'); session.scratch = 'xyz'; const sessionNamed = session.scratch; const sessionKey = session.key(0); session.clear(); document.getElementById('out').textContent = before + '|' + token + ':' + local.theme + ':' + sessionNamed + ':' + String(local.length) + ':' + String(local.key(0)) + ':' + String(session.length) + ':' + String(sessionKey);</script>"
                .to_string(),
        ),
        local_storage: {
            let mut storage = BTreeMap::new();
            storage.insert("token".to_string(), "abc".to_string());
            storage
        },
    })
    .expect("session should expose web storage");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "[object Storage]:[object Storage]:1:0|abc:dark:xyz:1:theme:0:scratch"
    );
    assert_eq!(
        session
            .mocks()
            .storage()
            .local()
            .get("theme")
            .map(String::as_str),
        Some("dark")
    );
    assert!(session.mocks().storage().session().is_empty());
}

#[test]
fn session_reports_script_errors_from_inline_bootstrap() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>document.getElementById('missing').textContent = 'Hello';</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("missing elements should fail script bootstrap");

    assert!(error.to_string().contains("Script error"));
    assert!(
        error
            .to_string()
            .contains("document.getElementById(\"missing\") returned no element")
    );
}

#[test]
fn session_resolves_query_selector_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root' class='primary'>scope<section><div class='primary'>inside</div></section></main><div id='out'></div><script>const docMatch = document.querySelector('.primary'); const scopedMatch = document.getElementById('root').querySelector('.primary'); const missing = document.getElementById('root').querySelector('.missing'); document.getElementById('out').textContent = docMatch.textContent + ':' + scopedMatch.textContent + ':' + String(missing);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute querySelector scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "scopeinside:inside:null"
    );
}

#[test]
fn session_resolves_query_selector_all_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root' class='primary'>root<section><div class='primary'>inside</div></section></main><div id='out'></div><script>const all = document.querySelectorAll('.primary'); const scoped = document.getElementById('root').querySelectorAll('.primary'); document.getElementById('out').textContent = String(all.length) + ':' + all.item(0).textContent + ':' + all.item(1).textContent + ':' + String(scoped.length) + ':' + scoped.item(0).textContent;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute querySelectorAll scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "2:rootinside:inside:1:inside"
    );
}

#[test]
fn session_resolves_scope_selectors_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><section id='section'><div id='child'>Child</div></section></main><div id='out'></div><script>const docScope = document.querySelector(':scope'); const root = document.getElementById('root'); const section = root.querySelector(':scope > section'); const missing = root.querySelector(':scope'); const matches = root.matches(':scope'); const closest = document.getElementById('child').closest(':scope'); document.getElementById('out').textContent = docScope.getAttribute('id') + ':' + section.getAttribute('id') + ':' + String(missing) + ':' + String(matches) + ':' + closest.getAttribute('id');</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute :scope selector scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "root:section:null:true:child"
    );
}

#[test]
fn session_resolves_has_selectors_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><section id='first' class='child'>First</section><section id='child' class='child'><div class='grandchild'>Grand</div></section></main><div id='out'></div><script>const docMatch = document.querySelector('main:has(#child)'); const directMatch = document.querySelector('main:has(> .child)'); const nthMatch = document.querySelector('main:has(:nth-child(2 of .child))'); const root = document.getElementById('root'); const section = document.getElementById('child'); const nested = document.querySelector('main:has(section .grandchild)'); const closest = section.closest('main:has(> .child)'); document.getElementById('out').textContent = docMatch.getAttribute('id') + ':' + directMatch.getAttribute('id') + ':' + nthMatch.getAttribute('id') + ':' + String(root.matches('main:has(> .child)')) + ':' + String(section.matches(':has(.grandchild)')) + ':' + closest.getAttribute('id') + ':' + nested.getAttribute('id');</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute :has selector scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "root:root:root:true:true:root:root"
    );
}

#[test]
fn session_resolves_attribute_value_selectors_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root' data-kind='APP-shell' lang='EN-US'><button id='first' data-role='Primary Action' data-tags='Primary Ready' data-label='Primary Action'>First</button><button id='second' data-role='Secondary Action'>Second</button><input id='toggle' disabled></main><div id='out'></div><script>const prefix = document.querySelector(\"button[data-role^=prim i]\"); const strict = document.querySelector(\"button[data-role^='Primary' s]\"); const suffix = document.querySelector(\"[data-label$='action' i]\"); const contains = document.querySelector(\"button[data-role*='ond' i]\"); const token = document.querySelector(\"[data-tags~=ready i]\"); const all = document.querySelectorAll(\"main[data-kind|=app i], button[data-role$='Action' s]\"); const second = document.getElementById('second'); const root = second.closest(\"main:is([lang|=en i], .blocked)\"); const disabled = document.querySelector(\"input[disabled='']\"); document.getElementById('out').textContent = prefix.textContent + ':' + strict.textContent + ':' + suffix.textContent + ':' + contains.textContent + ':' + token.textContent + ':' + String(all.length) + ':' + String(second.matches(\"button[data-role~=secondary i]\")) + ':' + root.textContent + ':' + String(disabled);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute attribute selector scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "First:First:First:Second:First:3:true:FirstSecond:[object Element]"
    );
}

#[test]
fn session_resolves_class_views_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><button id='button' class='base' data-kind='App'>First</button><div id='out'></div><script>const button = document.getElementById('button'); button.className = 'primary secondary'; const before = button.classList.length; const contains = button.classList.contains('primary'); button.classList.add('tertiary'); button.classList.remove('secondary'); const toggled = button.classList.toggle('active'); button.dataset.userId = '42'; const out = document.getElementById('out'); out.textContent = button.className + ':' + String(before) + ':' + String(contains) + ':' + String(toggled) + ':' + button.dataset.kind + ':' + button.dataset.userId + ':' + String(button.classList) + ':' + button.classList.toString() + ':' + String(button.classList.item(1)) + ':' + String(button.classList.keys().next().value) + ':' + String(button.classList.values().next().value) + ':' + String(button.classList.entries().next().value.index) + ':' + String(button.classList.entries().next().value.value) + ':' + String(button.dataset); button.classList.forEach((token, index, list) => { out.textContent += '|F' + String(index) + ':' + token + ':' + String(list.length); });</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute class view scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "primary tertiary active:2:true:true:App:42:[object DOMTokenList]:primary tertiary active:tertiary:0:primary:0:primary:[object DOMStringMap]|F0:primary:3|F1:tertiary:3|F2:active:3"
    );
    assert_eq!(session.dom().select(".active").unwrap().len(), 1);
    assert_eq!(session.dom().select("[data-user-id]").unwrap().len(), 1);
    assert_eq!(session.dom().select("[data-kind=App]").unwrap().len(), 1);
}

#[test]
fn session_supports_classlist_replace_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><button id='button' class='base primary active'>First</button><div id='out'></div><script>const button = document.getElementById('button'); const first = button.classList.replace('primary', 'secondary'); const second = button.classList.replace('missing', 'delta'); const third = button.classList.replace('secondary', 'active'); document.getElementById('out').textContent = button.className + ':' + String(first) + ':' + String(second) + ':' + String(third) + ':' + String(button.classList.length) + ':' + button.classList.toString();</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("classList.replace should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "base active:true:false:true:2:base active"
    );
    assert_eq!(session.dom().select("#button").unwrap().len(), 1);
}

#[test]
fn session_supports_classlist_value_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><button id='button' class='primary secondary'>First</button><div id='out'></div><script>const button = document.getElementById('button'); const before = button.classList.value; button.classList.value = 'alpha beta'; document.getElementById('out').textContent = before + ':' + button.className + ':' + button.classList.value + ':' + String(button.classList.length) + ':' + String(button.classList.contains('alpha')) + ':' + String(button.classList.contains('beta'));</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("classList.value should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "primary secondary:alpha beta:alpha beta:2:true:true"
    );
    assert_eq!(session.dom().select("#button").unwrap().len(), 1);
}

#[test]
fn session_resolves_not_selectors_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root' class='app' data-kind='APP READY'><button id='first' class='primary'>First</button><button id='disabled' class='primary' disabled>Disabled</button><button id='enabled' class='secondary'>Enabled</button><div id='out'></div><script>const enabled = document.querySelectorAll('button:not(:disabled)'); const second = document.getElementById('enabled'); const root = second.closest('main:not([data-kind~=blocked i], .blocked)'); const bounded = document.querySelectorAll('button:not(main > .secondary, :disabled)'); document.getElementById('out').textContent = String(enabled.length) + ':' + enabled.item(0).textContent + ':' + enabled.item(1).textContent + ':' + String(second.matches('button:not(.primary)')) + ':' + String(root.matches('main:not([data-kind~=blocked i], .blocked)')) + ':' + document.querySelector('button:not(:nth-child(even))').textContent + ':' + String(bounded.length) + ':' + bounded.item(0).textContent;</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute :not selector scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "2:First:Enabled:true:true:First:1:First"
    );
}

#[test]
fn session_resolves_is_selectors_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root' class='app' data-kind='APP READY' lang='EN-US'><button id='first' class='primary'>First</button><button id='disabled' class='primary' disabled>Disabled</button><button id='enabled' class='secondary'>Enabled</button></main><div id='out'></div><script>const all = document.querySelectorAll('button:is(.primary, .secondary)'); const filtered = document.querySelectorAll('button:is(:disabled, .secondary)'); const bounded = document.querySelectorAll('button:is(main > .secondary, :disabled)'); const second = document.getElementById('enabled'); const root = second.closest('main:is([lang|=en i], .blocked)'); document.getElementById('out').textContent = String(all.length) + ':' + String(filtered.length) + ':' + String(second.matches('button:is(.secondary, .blocked)')) + ':' + String(root.matches('main:is([lang|=en i], .blocked)')) + ':' + document.querySelector('button:is(.primary, .secondary):not(:disabled)').textContent + ':' + String(bounded.length) + ':' + bounded.item(0).textContent;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute :is selector scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "3:2:true:true:First:2:Disabled"
    );
}

#[test]
fn session_resolves_where_selectors_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root' class='app' data-kind='APP READY' lang='EN-US'><button id='first' class='primary'>First</button><button id='disabled' class='primary' disabled>Disabled</button><button id='enabled' class='secondary'>Enabled</button></main><div id='out'></div><script>const all = document.querySelectorAll('button:where(.primary, .secondary)'); const filtered = document.querySelectorAll('button:where(:disabled, .secondary)'); const bounded = document.querySelectorAll('button:where(main > .secondary, :disabled)'); const second = document.getElementById('enabled'); const root = second.closest('main:where([lang|=en i], .blocked)'); document.getElementById('out').textContent = String(all.length) + ':' + String(filtered.length) + ':' + String(second.matches('button:where(.secondary, .blocked)')) + ':' + String(root.matches('main:where([lang|=en i], .blocked)')) + ':' + document.querySelector('button:where(.primary, .secondary):not(:disabled)').textContent + ':' + String(bounded.length) + ':' + bounded.item(0).textContent;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute :where selector scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "3:2:true:true:First:2:Disabled"
    );
}

#[test]
fn session_resolves_nth_last_child_selectors_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main>lead<!-- gap --><button id='first' class='primary'>First</button><button id='disabled' class='primary' disabled>Disabled</button><button id='enabled' class='primary'>Enabled</button><input id='agree' type='checkbox' checked><select id='mode'><option value='a'>A</option><option id='selected' value='b' selected>B</option></select></main><div id='out'></div><script>const first = document.querySelector('button:nth-last-child(5)'); const second = document.querySelector('button:nth-last-child(4)'); const odd = document.querySelectorAll('button:nth-last-child(odd)'); const even = document.querySelector('button:nth-last-child(even)'); const formula = document.querySelector('button:nth-last-child(2n+1)'); document.getElementById('out').textContent = first.textContent + ':' + second.textContent + ':' + String(odd.length) + ':' + even.textContent + ':' + formula.textContent;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute nth-last-child selector scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "First:Disabled:2:Disabled:First"
    );
}

#[test]
fn session_resolves_element_children_html_collection_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><span id='first'>First</span><span id='second'>Second</span></main><div id='out'></div><script>const children = document.getElementById('root').children; const before = children.length; document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(before) + ':' + String(children.length) + ':' + String(children.item(0));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute HTMLCollection scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "2:0:null");
}

#[test]
fn session_resolves_element_children_html_collection_named_item_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><span name='alpha'>First</span><span id='second'>Second</span></main><div id='out'></div><script>const children = document.getElementById('root').children; const alpha = children.namedItem('alpha'); const second = children.namedItem('second'); document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = alpha.textContent + ':' + second.textContent + ':' + String(children.namedItem('alpha'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute HTMLCollection namedItem scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "First:Second:null"
    );
}

#[test]
fn session_resolves_get_elements_by_tag_name_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><span name='alpha'>First</span><span id='second'>Second</span></main><div id='out'></div><script>const all = document.getElementsByTagName('span'); const scoped = document.getElementById('root').getElementsByTagName('span'); const alpha = all.namedItem('alpha'); const second = scoped.namedItem('second'); const before = all.length; const beforeScoped = scoped.length; document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(before) + ':' + String(all.length) + ':' + String(beforeScoped) + ':' + String(scoped.length) + ':' + alpha.textContent + ':' + second.textContent + ':' + String(all.namedItem('alpha'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute getElementsByTagName scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "2:0:2:0:First:Second:null"
    );
}

#[test]
fn session_resolves_get_elements_by_class_name_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root' class='alpha'><span name='alpha' class='alpha'>First</span><span id='second' class='alpha'>Second</span></main><div id='out'></div><script>const all = document.getElementsByClassName('alpha'); const scoped = document.getElementById('root').getElementsByClassName('alpha'); const named = all.namedItem('alpha'); const root = all.item(0); const before = all.length; const beforeScoped = scoped.length; document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(before) + ':' + String(all.length) + ':' + String(beforeScoped) + ':' + String(scoped.length) + ':' + named.textContent + ':' + String(scoped.namedItem('alpha')) + ':' + root.textContent;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute getElementsByClassName scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "3:1:2:0:First:null:gone"
    );
}

#[test]
fn session_resolves_get_elements_by_name_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><span name='alpha'>First</span><span name='alpha'>Second</span></main><div id='out'></div><script>const nodes = document.getElementsByName('alpha'); const first = nodes.item(0); const before = nodes.length; document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(before) + ':' + String(nodes.length) + ':' + first.textContent + ':' + String(nodes.item(1));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute getElementsByName scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "2:0:First:null"
    );
}

#[test]
fn session_resolves_document_forms_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><form id='signup' name='signup'>Signup</form><form id='login' name='login'>Login</form></div><div id='out'></div><script>const forms = document.forms; const first = forms.item(0); const named = forms.namedItem('signup'); const signup = forms.signup; const login = forms.login; const before = forms.length; const firstText = first.textContent; const namedText = named.textContent; document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(before) + ':' + String(forms.length) + ':' + firstText + ':' + namedText + ':' + signup.textContent + ':' + login.textContent + ':' + String(forms.namedItem('missing'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute document.forms scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "2:0:Signup:Signup:Signup:Login:null"
    );
}

#[test]
fn session_resolves_document_forms_iterator_helpers() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><form id='first-form' name='first-form'>First</form><form id='second-form' name='second-form'>Second</form></div><div id='out'></div><script>const forms = document.forms; const keys = forms.keys(); const values = forms.values(); const entries = forms.entries(); const firstKey = keys.next(); const firstValue = values.next(); const firstEntry = entries.next(); let out = ''; forms.forEach((element, index, list) => { out += String(index) + ':' + element.getAttribute('id') + ':' + String(list.length) + ';'; }); document.getElementById('out').textContent = String(firstKey.value) + ':' + firstValue.value.getAttribute('id') + ':' + String(firstEntry.value.index) + ':' + firstEntry.value.value.getAttribute('id') + ':' + out;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute document.forms iterator helper scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "0:first-form:0:first-form:0:first-form:2;1:second-form:2;"
    );
}

#[test]
fn session_resolves_form_elements_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><form id='signup'><input name='first' value='Ada'><textarea name='bio'>Bio</textarea></form></div><div id='out'></div><script>const elements = document.getElementById('signup').elements; const first = elements.item(0); const named = elements.namedItem('first'); const before = elements.length; const firstValue = first.value; const namedValue = named.value; document.getElementById('signup').textContent = 'gone'; document.getElementById('out').textContent = String(before) + ':' + String(elements.length) + ':' + firstValue + ':' + namedValue + ':' + String(elements.namedItem('missing'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute form elements scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "2:0:Ada:Ada:null"
    );
}

#[test]
fn session_resolves_form_elements_radio_node_list_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><form id='signup'><input type='radio' name='mode' id='mode-a' value='a'><input type='radio' name='mode' id='mode-b' value='b'><textarea name='bio'>Bio</textarea></form></div><div id='out'></div><script>const elements = document.getElementById('signup').elements; const named = elements.namedItem('mode'); const before = named.length; document.getElementById('signup').innerHTML += '<input type=\"radio\" name=\"mode\" id=\"mode-c\" value=\"c\" checked>'; document.getElementById('out').textContent = String(before) + ':' + String(named.length) + ':' + named.item(0).value + ':' + named.item(1).value + ':' + named.value + ':' + String(named);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute radio node list scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "2:3:a:b:c:[object RadioNodeList]"
    );
}

#[test]
fn session_resolves_html_collection_named_property_access_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><span id='first'>First</span><form id='signup'><input type='radio' name='mode' id='mode-a' value='a'><input type='radio' name='mode' id='mode-b' value='b'></form></div><div id='out'></div><script>const children = document.getElementById('root').children; const mode = document.getElementById('signup').elements.mode; document.getElementById('out').textContent = children.first.textContent + ':' + String(children.missing) + ':' + String(mode.length) + ':' + mode.item(0).value + ':' + mode.item(1).value + ':' + String(mode);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose HTMLCollection named property access");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "First:undefined:2:a:b:[object RadioNodeList]"
    );
}

#[test]
fn session_resolves_radio_node_list_iterator_helpers() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><form id='signup'><input type='radio' name='mode' id='mode-a' value='a'><input type='radio' name='mode' id='mode-b' value='b'></form></div><div id='out'></div><script>const elements = document.getElementById('signup').elements; const named = elements.namedItem('mode'); const keys = named.keys(); const values = named.values(); const entries = named.entries(); const firstKey = keys.next(); const firstValue = values.next(); const firstEntry = entries.next(); let out = ''; named.forEach((element, index, list) => { out += String(index) + ':' + element.value + ':' + String(list.length) + ';'; }); document.getElementById('out').textContent = String(named.length) + ':' + String(firstKey.value) + ':' + firstValue.value.value + ':' + String(firstEntry.value.index) + ':' + firstEntry.value.value.value + ':' + out;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("radio node list iterator helpers should remain available");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "2:0:a:0:a:0:a:2;1:b:2;"
    );
}

#[test]
fn session_sets_radio_node_list_value_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><form id='signup'><input type='radio' name='mode' id='mode-a' value='a'><input type='radio' name='mode' id='mode-b' value='b'><input type='radio' name='mode' id='mode-c' value='c'></form></div><div id='out'></div><script>const named = document.getElementById('signup').elements.namedItem('mode'); named.value = 'b'; document.getElementById('out').textContent = named.value + ':' + String(document.getElementById('mode-a').checked) + ':' + String(document.getElementById('mode-b').checked) + ':' + String(document.getElementById('mode-c').checked);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute radio node list value scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "b:false:true:false"
    );
}

#[test]
fn session_resolves_select_options_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><select id='mode'><option name='alpha' value='a'>A</option><option id='second' value='b'>B</option></select></div><div id='out'></div><script>const options = document.getElementById('mode').options; const first = options.item(0); const named = options.namedItem('second'); const before = options.length; const firstText = first.textContent; const namedText = named.textContent; document.getElementById('mode').textContent = 'gone'; document.getElementById('out').textContent = String(before) + ':' + String(options.length) + ':' + firstText + ':' + namedText + ':' + String(options.namedItem('missing'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute select.options scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "2:0:A:B:null");
}

#[test]
fn session_resolves_select_options_iterator_helpers_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><select id='mode'><option id='first' value='a'>A</option><option id='second' value='b'>B</option></select></div><div id='out'></div><script>const options = document.getElementById('mode').options; const keys = options.keys(); const values = options.values(); const entries = options.entries(); const firstKey = keys.next(); const firstValue = values.next(); const firstEntry = entries.next(); let out = ''; options.forEach((element, index, list) => { out += String(index) + ':' + element.getAttribute('id') + ':' + String(list.length) + ';'; }); document.getElementById('out').textContent = String(firstKey.value) + ':' + firstValue.value.getAttribute('id') + ':' + String(firstEntry.value.index) + ':' + firstEntry.value.value.getAttribute('id') + ':' + out;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute select.options iterator helper scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "0:first:0:first:0:first:2;1:second:2;"
    );
}

#[test]
fn session_adds_and_removes_select_options_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><select id='mode'><option id='first' value='a'>A</option></select><option id='extra' value='b'>B</option></div><div id='out'></div><script>const select = document.getElementById('mode'); const extra = document.getElementById('extra'); const before = select.options.length; select.options.add(extra); const afterAdd = select.options.length; select.options.remove(0); document.getElementById('out').textContent = String(before) + ':' + String(afterAdd) + ':' + String(select.options.length) + ':' + select.options.item(0).getAttribute('id') + ':' + String(select.options.namedItem('first'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute select.options mutation scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "1:2:1:extra:null"
    );
    assert_eq!(session.dom().select("#extra").unwrap().len(), 1);
    assert!(session.dom().select("#first").unwrap().is_empty());
}

#[test]
fn session_resolves_select_selected_options_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><select id='mode'><option id='first' value='a' selected>A</option><option id='second' value='b'>B</option></select></div><div id='out'></div><script>const select = document.getElementById('mode'); const selected = select.selectedOptions; const before = selected.length; const first = selected.item(0); select.innerHTML = '<option id=\"third\" value=\"c\" selected>C</option><option id=\"fourth\" value=\"d\" selected>D</option>'; document.getElementById('out').textContent = String(before) + ':' + String(selected.length) + ':' + first.textContent + ':' + selected.item(0).textContent + ':' + selected.item(1).textContent + ':' + String(selected.namedItem('third')) + ':' + String(selected.namedItem('missing'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute select.selectedOptions scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "1:2:A:C:D:[object Element]:null"
    );
}

#[test]
fn session_resolves_select_selected_options_iterator_helpers_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><select id='mode'><option id='first' value='a' selected>A</option><option id='second' value='b' selected>B</option></select></div><div id='out'></div><script>const selected = document.getElementById('mode').selectedOptions; const keys = selected.keys(); const values = selected.values(); const entries = selected.entries(); const firstKey = keys.next(); const firstValue = values.next(); const firstEntry = entries.next(); let out = ''; selected.forEach((element, index, list) => { out += String(index) + ':' + element.getAttribute('id') + ':' + String(list.length) + ';'; }); document.getElementById('out').textContent = String(firstKey.value) + ':' + firstValue.value.getAttribute('id') + ':' + String(firstEntry.value.index) + ':' + firstEntry.value.value.getAttribute('id') + ':' + out;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute select.selectedOptions iterator helper scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "0:first:0:first:0:first:2;1:second:2;"
    );
}

#[test]
fn session_resolves_option_selected_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><select id='mode'><option id='first' value='a'>A</option><option id='second' value='b'>B</option></select></div><div id='out'></div><script>const second = document.getElementById('second'); const before = second.selected; second.selected = true; const selected = document.getElementById('mode').selectedOptions; document.getElementById('out').textContent = String(before) + ':' + String(second.selected) + ':' + String(selected.length) + ':' + selected.item(0).getAttribute('id') + ':' + document.querySelector('option:checked').getAttribute('id');</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute option.selected scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "false:true:1:second:second"
    );
}

#[test]
fn session_resolves_option_default_selected_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><select id='mode'><option id='first' value='a'>A</option><option id='second' value='b'>B</option></select></div><div id='out'></div><script>const second = document.getElementById('second'); const before = second.defaultSelected; second.defaultSelected = true; const selected = document.getElementById('mode').selectedOptions; document.getElementById('out').textContent = String(before) + ':' + String(second.defaultSelected) + ':' + String(selected.length) + ':' + selected.item(0).getAttribute('id') + ':' + document.querySelector('option:checked').getAttribute('id');</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute option.defaultSelected scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "false:true:1:second:second"
    );
}

#[test]
fn session_resolves_select_selected_index_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><select id='mode'><option id='first' value='a' selected>A</option><option id='second' value='b'>B</option><option id='third' value='c'>C</option></select></div><div id='out'></div><script>const select = document.getElementById('mode'); const before = select.selectedIndex; select.selectedIndex = 2; document.getElementById('out').textContent = String(before) + ':' + String(select.selectedIndex) + ':' + String(document.querySelector('option:checked').getAttribute('id'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute select.selectedIndex scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "0:2:third");
}

#[test]
fn session_resolves_select_value_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><select id='mode'><option id='first' value='a' selected>A</option><option id='second' value='b'>B</option></select></div><div id='out'></div><script>const select = document.getElementById('mode'); const before = select.value; select.value = 'b'; const afterMatch = select.value; select.value = 'missing'; const afterMissing = select.value; document.getElementById('out').textContent = before + ':' + afterMatch + ':' + afterMissing + ':' + String(select.selectedIndex) + ':' + String(document.querySelectorAll('option:checked').length) + ':' + String(document.querySelector('option:checked'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute select.value scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "a:b::-1:0:null"
    );
    assert_eq!(session.dom().select("option:checked").unwrap().len(), 0);
    assert_eq!(session.dom().select("#first[selected]").unwrap().len(), 0);
    assert_eq!(session.dom().select("#second[selected]").unwrap().len(), 0);
}

#[test]
fn session_resolves_select_type_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><select id='mode'><option id='first' value='a'>A</option></select></div><div id='out'></div><script>const select = document.getElementById('mode'); const before = select.type; select.multiple = true; const after = select.type; document.getElementById('out').textContent = before + ':' + after;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute select.type scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "select-one:select-multiple"
    );
}

#[test]
fn session_resolves_option_index_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><select id='mode'><option id='first' value='a'>A</option><option id='second' value='b'>B</option></select></div><div id='out'></div><script>const select = document.getElementById('mode'); const second = document.getElementById('second'); const before = second.index; select.insertAdjacentHTML('afterbegin', '<option id=\"zero\" value=\"z\">Z</option>'); document.getElementById('out').textContent = String(before) + ':' + String(second.index) + ':' + String(document.getElementById('zero').index);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute option.index scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "1:2:0");
}

#[test]
fn session_resolves_option_form_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><form id='owner'><select id='mode'><option id='first' value='a'>A</option><option id='second' value='b'>B</option></select></form></div><div id='out'></div><script>const second = document.getElementById('second'); const before = second.form; document.getElementById('mode').remove(); document.getElementById('out').textContent = before.getAttribute('id') + ':' + String(second.form);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute option.form scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "owner:null");
}

#[test]
fn session_resolves_button_form_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><form id='owner'><button id='button'>Button</button></form></div><div id='out'></div><script>const button = document.getElementById('button'); document.getElementById('out').textContent = button.form.getAttribute('id');</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute button.form scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "owner");
}

#[test]
fn session_resolves_fieldset_and_output_form_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><form id='owner'><fieldset id='fieldset'></fieldset><output id='output'></output></form></div><div id='out'></div><script>const fieldset = document.getElementById('fieldset'); const output = document.getElementById('output'); const beforeFieldset = fieldset.form; const beforeOutput = output.form; document.getElementById('owner').removeChild(fieldset); document.getElementById('owner').removeChild(output); document.getElementById('out').textContent = beforeFieldset.getAttribute('id') + ':' + beforeOutput.getAttribute('id') + ':' + String(fieldset.form) + ':' + String(output.form);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute fieldset.form and output.form scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "owner:owner:null:null"
    );
}

#[test]
fn session_resolves_object_form_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><form id='owner'><object id='asset'></object></form></div><div id='out'></div><script>const asset = document.getElementById('asset'); const before = asset.form; document.getElementById('owner').removeChild(asset); document.getElementById('out').textContent = before.getAttribute('id') + ':' + String(asset.form);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute object.form scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "owner:null");
}

#[test]
fn session_resolves_embed_form_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><form id='owner'><embed id='asset'></form></div><div id='out'></div><script>const asset = document.getElementById('asset'); const before = asset.form; document.getElementById('owner').removeChild(asset); document.getElementById('out').textContent = before.getAttribute('id') + ':' + String(asset.form);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute embed.form scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "owner:null");
}

#[test]
fn session_resolves_option_disabled_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><select id='mode'><option id='second' value='b'>B</option></select></div><div id='out'></div><script>const second = document.getElementById('second'); const before = second.disabled; second.disabled = true; const afterSet = second.disabled; second.disabled = false; const afterClear = second.disabled; document.getElementById('out').textContent = String(before) + ':' + String(afterSet) + ':' + String(afterClear) + ':' + String(document.querySelector('option:disabled'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute option.disabled scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "false:true:false:null"
    );
}

#[test]
fn session_resolves_form_control_disabled_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><input id='name'><button id='action'>Go</button></div><div id='out'></div><script>const input = document.getElementById('name'); const button = document.getElementById('action'); const before = String(input.disabled) + '|' + String(button.disabled) + '|' + String(document.querySelectorAll(':disabled').length) + '|' + String(document.querySelectorAll(':enabled').length); input.disabled = true; button.disabled = true; const afterSet = String(input.disabled) + '|' + String(button.disabled) + '|' + String(document.querySelectorAll(':disabled').length) + '|' + String(document.querySelectorAll(':enabled').length); input.disabled = false; button.disabled = false; const afterClear = String(input.disabled) + '|' + String(button.disabled) + '|' + String(document.querySelectorAll(':disabled').length) + '|' + String(document.querySelectorAll(':enabled').length); document.getElementById('out').textContent = before + ';' + afterSet + ';' + afterClear;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute disabled scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "false|false|0|2;true|true|2|0;false|false|0|2"
    );
}

#[test]
fn session_resolves_option_label_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><select id='mode'><option id='second' value='b'>B</option></select></div><div id='out'></div><script>const second = document.getElementById('second'); const before = second.label; second.label = 'Bee'; const afterSet = second.label; document.getElementById('out').textContent = String(before) + ':' + String(afterSet) + ':' + String(second.textContent) + ':' + String(document.querySelector('option[label=Bee]').getAttribute('id'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute option.label scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "B:Bee:B:second"
    );
}

#[test]
fn session_resolves_option_text_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><select id='mode'><option id='second' value='b'>B</option></select><div id='out'></div><script>const second = document.getElementById('second'); const before = second.text; second.text = 'Bee'; const afterSet = second.text; document.getElementById('out').textContent = String(before) + ':' + String(afterSet) + ':' + String(second.textContent) + ':' + String(second.label);</script></div>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute option.text scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "B:Bee:Bee:Bee");
}

#[test]
fn session_resolves_select_multiple_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><select id='mode'><option id='second' value='b'>B</option></select><div id='out'></div><script>const select = document.getElementById('mode'); const before = select.multiple; select.multiple = true; const afterSet = select.multiple; const afterSetAttr = select.getAttribute('multiple'); select.multiple = false; const afterClear = select.multiple; document.getElementById('out').textContent = String(before) + ':' + String(afterSet) + ':' + String(afterSetAttr) + ':' + String(afterClear);</script></div>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute select.multiple scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "false:true::false"
    );
}

#[test]
fn session_resolves_select_size_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><select id='mode'></select></div><div id='out'></div><script>const select = document.getElementById('mode'); const before = select.size; select.size = 3; const afterSet = select.size; const afterSetAttr = select.getAttribute('size'); select.size = 0; const afterZero = select.size; const afterZeroAttr = select.getAttribute('size'); document.getElementById('out').textContent = String(before) + ':' + String(afterSet) + ':' + String(afterSetAttr) + ':' + String(afterZero) + ':' + String(afterZeroAttr);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute select.size scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "0:3:3:0:0");
}

#[test]
fn session_resolves_select_required_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><select id='mode'></select></div><div id='out'></div><script>const select = document.getElementById('mode'); const before = select.required; select.required = true; const afterSet = select.required; const afterSetAttr = select.getAttribute('required'); select.required = false; const afterClear = select.required; document.getElementById('out').textContent = String(before) + ':' + String(afterSet) + ':' + String(afterSetAttr) + ':' + String(afterClear) + ':' + String(document.querySelector('select:required'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute select.required scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "false:true::false:null"
    );
}

#[test]
fn session_resolves_form_no_validate_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><form id='signup'><button id='submit'>Submit</button><input id='field'></form></div><div id='out'></div><script>const form = document.getElementById('signup'); const button = document.getElementById('submit'); const field = document.getElementById('field'); const before = String(form.noValidate) + '|' + String(button.formNoValidate) + '|' + String(field.formNoValidate); form.noValidate = true; button.formNoValidate = true; field.formNoValidate = true; const afterSet = String(form.noValidate) + '|' + String(button.formNoValidate) + '|' + String(field.formNoValidate) + '|' + String(form.getAttribute('novalidate')) + '|' + String(button.getAttribute('formnovalidate')) + '|' + String(field.getAttribute('formnovalidate')); form.noValidate = false; button.formNoValidate = false; field.formNoValidate = false; const afterClear = String(form.noValidate) + '|' + String(button.formNoValidate) + '|' + String(field.formNoValidate) + '|' + String(form.hasAttribute('novalidate')) + '|' + String(button.hasAttribute('formnovalidate')) + '|' + String(field.hasAttribute('formnovalidate')); document.getElementById('out').textContent = before + ';' + afterSet + ';' + afterClear;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute noValidate scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "false|false|false;true|true|true|||;false|false|false|false|false|false"
    );
}

#[test]
fn session_resolves_form_submission_metadata_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><form id='signup' action='/submit?from=form#frag' method='BOGUS' enctype='BOGUS'><button id='submit'>Submit</button><input id='field' type='submit' formaction='/field-preview?x=1#field'></form></div><div id='out'></div><script>const form = document.getElementById('signup'); const button = document.getElementById('submit'); const field = document.getElementById('field'); const before = String(form.action) + '|' + String(button.formAction) + '|' + String(field.formAction) + '|' + String(form.method) + '|' + String(form.enctype) + '|' + String(form.target) + '|' + String(button.formMethod) + '|' + String(button.formEnctype) + '|' + String(button.formTarget) + '|' + String(field.formMethod) + '|' + String(field.formEnctype) + '|' + String(field.formTarget); form.action = '/override-form?x=1#form'; form.method = 'Dialog'; form.enctype = 'Multipart/Form-Data'; form.target = '_blank'; button.formAction = '/override-button?button=1#button'; button.formMethod = 'PoSt'; button.formEnctype = 'Text/Plain'; button.formTarget = 'preview'; field.formAction = '/override-field?field=1#field'; field.formMethod = 'GET'; field.formEnctype = 'application/x-www-form-urlencoded'; field.formTarget = 'field-preview'; const afterSet = String(form.action) + '|' + String(button.formAction) + '|' + String(field.formAction) + '|' + String(form.method) + '|' + String(form.enctype) + '|' + String(form.target) + '|' + String(button.formMethod) + '|' + String(button.formEnctype) + '|' + String(button.formTarget) + '|' + String(field.formMethod) + '|' + String(field.formEnctype) + '|' + String(field.formTarget) + '|' + String(form.getAttribute('action')) + '|' + String(form.getAttribute('method')) + '|' + String(form.getAttribute('enctype')) + '|' + String(form.getAttribute('target')) + '|' + String(button.getAttribute('formaction')) + '|' + String(button.getAttribute('formmethod')) + '|' + String(button.getAttribute('formenctype')) + '|' + String(button.getAttribute('formtarget')) + '|' + String(field.getAttribute('formaction')) + '|' + String(field.getAttribute('formmethod')) + '|' + String(field.getAttribute('formenctype')) + '|' + String(field.getAttribute('formtarget')); document.getElementById('out').textContent = before + ';' + afterSet;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute form submission metadata scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "https://example.test/submit?from=form#frag|https://example.test/submit?from=form#frag|https://example.test/field-preview?x=1#field|get|application/x-www-form-urlencoded||get|application/x-www-form-urlencoded||get|application/x-www-form-urlencoded|;https://example.test/override-form?x=1#form|https://example.test/override-button?button=1#button|https://example.test/override-field?field=1#field|dialog|multipart/form-data|_blank|post|text/plain|preview|get|application/x-www-form-urlencoded|field-preview|/override-form?x=1#form|dialog|multipart/form-data|_blank|/override-button?button=1#button|post|text/plain|preview|/override-field?field=1#field|get|application/x-www-form-urlencoded|field-preview"
    );
}

#[test]
fn session_rejects_non_form_submission_metadata_access_explicitly() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='wrapper'><div id='box'></div></div><script>document.getElementById('box').method;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-form submission metadata getter should fail explicitly");

    assert!(session.to_string().contains("method"));
}

#[test]
fn session_resolves_input_textarea_readonly_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><input id='name' value='Ada'><textarea id='bio'>Hello</textarea></div><div id='out'></div><script>const input = document.getElementById('name'); const textarea = document.getElementById('bio'); const before = String(input.readOnly) + '|' + String(textarea.readOnly); input.readOnly = true; textarea.readOnly = true; const afterSet = String(input.readOnly) + '|' + String(textarea.readOnly); const afterAttr = String(input.hasAttribute('readonly')) + '|' + String(textarea.hasAttribute('readonly')); const afterReadOnly = String(input.matches(':read-only')) + '|' + String(textarea.matches(':read-only')); input.readOnly = false; textarea.readOnly = false; const afterClear = String(input.readOnly) + '|' + String(textarea.readOnly); const afterClearReadOnly = String(input.matches(':read-only')) + '|' + String(textarea.matches(':read-only')); document.getElementById('out').textContent = before + ';' + afterSet + ';' + afterAttr + ';' + afterReadOnly + ';' + afterClear + ';' + afterClearReadOnly;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute readOnly scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "false|false;true|true;true|true;true|true;false|false;false|false"
    );
}

#[test]
fn session_resolves_input_textarea_autofocus_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><input id='name'><textarea id='bio'></textarea></div><div id='out'></div><script>const input = document.getElementById('name'); const textarea = document.getElementById('bio'); const before = String(input.autofocus) + '|' + String(textarea.autofocus) + '|' + String(document.querySelectorAll('[autofocus]').length); input.autofocus = true; textarea.autofocus = true; const afterSet = String(input.autofocus) + '|' + String(textarea.autofocus) + '|' + String(document.querySelectorAll('[autofocus]').length); input.autofocus = false; textarea.autofocus = false; const afterClear = String(input.autofocus) + '|' + String(textarea.autofocus) + '|' + String(document.querySelectorAll('[autofocus]').length); document.getElementById('out').textContent = before + ';' + afterSet + ';' + afterClear;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute autofocus scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "false|false|0;true|true|2;false|false|0"
    );
}

#[test]
fn session_resolves_input_textarea_autocomplete_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><input id='name'><textarea id='bio'></textarea></div><div id='out'></div><script>const input = document.getElementById('name'); const textarea = document.getElementById('bio'); const before = String(input.autocomplete) + '|' + String(textarea.autocomplete); input.autocomplete = 'email'; textarea.autocomplete = 'off'; const afterSet = String(input.autocomplete) + '|' + String(textarea.autocomplete); const afterAttr = String(input.getAttribute('autocomplete')) + '|' + String(textarea.getAttribute('autocomplete')); input.autocomplete = ''; textarea.autocomplete = ''; const afterClear = String(input.autocomplete) + '|' + String(textarea.autocomplete) + '|' + String(input.hasAttribute('autocomplete')) + '|' + String(textarea.hasAttribute('autocomplete')); document.getElementById('out').textContent = before + ';' + afterSet + ';' + afterAttr + ';' + afterClear;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute autocomplete scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "|;email|off;email|off;||true|true"
    );
}

#[test]
fn session_resolves_input_textarea_length_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><input id='name' minlength='2' maxlength='4' value='Ada'><textarea id='bio' minlength='2' maxlength='4'>Bio</textarea></div><div id='out'></div><script>const input = document.getElementById('name'); const textarea = document.getElementById('bio'); const before = String(input.minLength) + '|' + String(input.maxLength) + '|' + String(textarea.minLength) + '|' + String(textarea.maxLength) + '|' + String(document.querySelectorAll(':invalid').length); input.minLength = 4; textarea.maxLength = 2; const afterSet = String(input.minLength) + '|' + String(input.maxLength) + '|' + String(textarea.minLength) + '|' + String(textarea.maxLength) + '|' + String(document.querySelectorAll(':invalid').length); input.value = 'Jane'; textarea.value = 'No'; const afterValue = String(document.querySelectorAll(':invalid').length) + ':' + String(document.querySelectorAll(':valid').length); document.getElementById('out').textContent = before + ';' + afterSet + ';' + afterValue;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute length scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "2|4|2|4|0;4|4|2|2|2;0:2"
    );
}

#[test]
fn session_resolves_input_range_bounds_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><input id='low' type='number' value='1'><input id='high' type='number' value='4'></div><div id='out'></div><script>const low = document.getElementById('low'); const high = document.getElementById('high'); const before = String(low.min) + '|' + String(low.max) + '|' + String(high.min) + '|' + String(high.max) + '|' + String(document.querySelectorAll(':in-range').length) + '|' + String(document.querySelectorAll(':out-of-range').length); low.min = 2; low.max = 6; high.min = 2; high.max = 6; const afterSet = String(low.min) + '|' + String(low.max) + '|' + String(high.min) + '|' + String(high.max) + '|' + String(document.querySelectorAll(':in-range').length) + '|' + String(document.querySelectorAll(':out-of-range').length) + '|' + document.querySelectorAll(':in-range').item(0).id + '|' + document.querySelectorAll(':out-of-range').item(0).id; low.removeAttribute('min'); low.removeAttribute('max'); high.removeAttribute('min'); high.removeAttribute('max'); const afterClear = String(low.min) + '|' + String(low.max) + '|' + String(high.min) + '|' + String(high.max) + '|' + String(document.querySelectorAll(':in-range').length) + '|' + String(document.querySelectorAll(':out-of-range').length); document.getElementById('out').textContent = before + ';' + afterSet + ';' + afterClear;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute range bounds scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "||||0|0;2|6|2|6|1|1|high|low;||||0|0"
    );
}

#[test]
fn session_resolves_input_step_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><input id='speed' type='number' step='0.5'><div id='out'></div><script>const input = document.getElementById('speed'); const before = String(input.step); input.step = '2'; const afterSet = String(input.step) + '|' + String(input.getAttribute('step')); input.removeAttribute('step'); const afterClear = String(input.step) + '|' + String(input.hasAttribute('step')); document.getElementById('out').textContent = before + ';' + afterSet + ';' + afterClear;</script></div>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute step scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "0.5;2|2;|false"
    );
}

#[test]
fn session_resolves_input_size_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><input id='speed' type='text' size='5'><div id='out'></div><script>const input = document.getElementById('speed'); const before = String(input.size); input.size = 9; const afterSet = String(input.size) + '|' + String(input.getAttribute('size')); input.removeAttribute('size'); const afterClear = String(input.size) + '|' + String(input.hasAttribute('size')); document.getElementById('out').textContent = before + ';' + afterSet + ';' + afterClear;</script></div>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute size scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "5;9|9;20|false"
    );
}

#[test]
fn session_resolves_textarea_rows_and_cols_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><textarea id='note' rows='3' cols='40'>Hello</textarea><div id='out'></div><script>const textarea = document.getElementById('note'); const before = String(textarea.rows) + '|' + String(textarea.cols); textarea.rows = 7; textarea.cols = 50; const afterSet = String(textarea.rows) + '|' + String(textarea.cols) + '|' + String(textarea.getAttribute('rows')) + '|' + String(textarea.getAttribute('cols')); textarea.removeAttribute('rows'); textarea.removeAttribute('cols'); document.getElementById('out').textContent = before + ';' + afterSet + ';' + String(textarea.rows) + '|' + String(textarea.cols);</script></div>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute textarea rows/cols scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "3|40;7|50|7|50;2|20"
    );
}

#[test]
fn session_resolves_textarea_wrap_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><textarea id='note'>Hello</textarea><div id='out'></div><script>const textarea = document.getElementById('note'); const before = textarea.wrap; textarea.wrap = 'hard'; const afterSet = String(textarea.wrap) + '|' + String(textarea.getAttribute('wrap')); textarea.removeAttribute('wrap'); const afterClear = String(textarea.wrap) + '|' + String(textarea.getAttribute('wrap')); textarea.wrap = 'off'; const afterOff = String(textarea.wrap) + '|' + String(textarea.getAttribute('wrap')); document.getElementById('out').textContent = before + ':' + afterSet + ':' + afterClear + ':' + afterOff;</script></div>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute textarea.wrap scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "soft:hard|hard:soft|null:off|off"
    );
}

#[test]
fn session_resolves_input_pattern_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><input id='name' pattern='A[a-z]+' value='Ada'></div><div id='out'></div><script>const input = document.getElementById('name'); const before = String(input.pattern) + '|' + String(document.querySelectorAll(':invalid').length); input.pattern = 'B[a-z]+'; const afterSet = String(input.pattern) + '|' + String(input.getAttribute('pattern')) + '|' + String(document.querySelectorAll(':invalid').length); input.value = 'Bob'; const afterValue = String(document.querySelectorAll(':invalid').length) + ':' + String(document.querySelectorAll(':valid').length); document.getElementById('out').textContent = before + ';' + afterSet + ';' + afterValue;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute pattern scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "A[a-z]+|0;B[a-z]+|B[a-z]+|1;0:1"
    );
}

#[test]
fn session_resolves_input_textarea_placeholder_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><input id='name' placeholder='Name'><textarea id='bio' placeholder='Bio'></textarea></div><div id='out'></div><script>const input = document.getElementById('name'); const textarea = document.getElementById('bio'); const before = String(input.placeholder) + '|' + String(textarea.placeholder) + '|' + String(document.querySelectorAll(':placeholder-shown').length); input.placeholder = 'Full name'; textarea.placeholder = 'Biography'; const afterSet = String(input.placeholder) + '|' + String(textarea.placeholder) + '|' + String(input.getAttribute('placeholder')) + '|' + String(textarea.getAttribute('placeholder')) + '|' + String(document.querySelectorAll(':placeholder-shown').length); input.value = 'Alice'; textarea.value = 'Bio text'; const afterValue = String(document.querySelectorAll(':placeholder-shown').length); document.getElementById('out').textContent = before + ';' + afterSet + ';' + afterValue;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute placeholder scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "Name|Bio|2;Full name|Biography|Full name|Biography|2;0"
    );
}

#[test]
fn session_resolves_input_textarea_default_value_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><input id='name' value='Initial'><textarea id='bio'>Hello</textarea></div><div id='out'></div><script>const input = document.getElementById('name'); const textarea = document.getElementById('bio'); const before = String(input.defaultValue) + '|' + String(textarea.defaultValue); input.defaultValue = 'Updated'; textarea.defaultValue = 'World'; const afterSet = String(input.defaultValue) + '|' + String(textarea.defaultValue) + '|' + String(input.getAttribute('value')) + '|' + String(textarea.textContent); document.getElementById('out').textContent = before + ';' + afterSet;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute defaultValue scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "Initial|Hello;Updated|World|Updated|World"
    );
}

#[test]
fn session_resolves_blank_pseudo_class_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><input id='empty' type='text' value='  '><textarea id='bio'> \n </textarea><input id='check' type='checkbox'></main><div id='out'></div><script>const blank = document.querySelectorAll(':blank'); const empty = document.getElementById('empty'); const bio = document.getElementById('bio'); const check = document.getElementById('check'); document.getElementById('out').textContent = String(blank.length) + ':' + blank.item(0).getAttribute('id') + ':' + blank.item(1).getAttribute('id') + ':' + String(empty.matches(':blank')) + ':' + String(bio.matches(':blank')) + ':' + String(check.matches(':blank'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("blank selector should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "2:empty:bio:true:true:false"
    );
}

#[test]
fn session_rejects_non_input_range_bounds_explicitly() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some("<script>document.createElement('textarea').min = 1;</script>".to_string()),
        local_storage: BTreeMap::new(),
    })
    .expect_err("textarea min should be rejected");

    assert!(session.to_string().contains("min"));
}

#[test]
fn session_rejects_non_input_pattern_explicitly() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<script>document.createElement('textarea').pattern = 'abc';</script>".to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("textarea pattern should be rejected");

    assert!(session.to_string().contains("pattern"));
}

#[test]
fn session_rejects_non_input_step_explicitly() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some("<script>document.createElement('div').step;</script>".to_string()),
        local_storage: BTreeMap::new(),
    })
    .expect_err("div step should be rejected");

    assert!(session.to_string().contains("step"));
}

#[test]
fn session_rejects_non_input_size_explicitly() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some("<script>document.createElement('div').size;</script>".to_string()),
        local_storage: BTreeMap::new(),
    })
    .expect_err("div size should be rejected");

    assert!(session.to_string().contains("size"));
}

#[test]
fn session_rejects_non_textarea_cols_explicitly() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some("<script>document.createElement('div').cols;</script>".to_string()),
        local_storage: BTreeMap::new(),
    })
    .expect_err("div cols should be rejected");

    assert!(session.to_string().contains("cols"));
}

#[test]
fn session_rejects_non_option_selected_access_explicitly() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='wrapper'><button id='button'>Button</button></div><script>document.getElementById('button').selected;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-option selected access should fail explicitly");

    assert!(session.to_string().contains("selected"));
}

#[test]
fn session_rejects_non_select_multiple_access_explicitly() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='wrapper'><button id='button'>Button</button></div><script>document.getElementById('button').multiple;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-select multiple access should fail explicitly");

    assert!(session.to_string().contains("multiple"));
}

#[test]
fn session_rejects_non_select_size_access_explicitly() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='wrapper'><button id='button'>Button</button></div><script>document.getElementById('button').size;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-select size access should fail explicitly");

    assert!(session.to_string().contains("size"));
}

#[test]
fn session_rejects_non_form_control_required_access_explicitly() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='wrapper'><button id='button'>Button</button></div><script>document.getElementById('button').required;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-form-control required access should fail explicitly");

    assert!(session.to_string().contains("required"));
}

#[test]
fn session_rejects_non_form_novalidate_access_getter_explicitly() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='wrapper'><div id='box'></div></div><script>document.getElementById('box').noValidate;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-form noValidate getter should fail explicitly");

    assert!(session.to_string().contains("noValidate"));
}

#[test]
fn session_rejects_non_form_novalidate_access_explicitly() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='wrapper'><div id='box'></div></div><script>document.getElementById('box').noValidate = true;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-form noValidate setter should fail explicitly");

    assert!(session.to_string().contains("noValidate"));
}

#[test]
fn session_rejects_non_form_form_no_validate_access_explicitly() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='wrapper'><div id='box'></div></div><script>document.getElementById('box').formNoValidate = true;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-form formNoValidate setter should fail explicitly");

    assert!(session.to_string().contains("formNoValidate"));
}

#[test]
fn session_rejects_non_form_control_readonly_access_explicitly() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='wrapper'><button id='button'>Button</button></div><script>document.getElementById('button').readOnly;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-form-control readOnly access should fail explicitly");

    assert!(session.to_string().contains("readOnly"));
}

#[test]
fn session_rejects_non_form_control_autofocus_access_explicitly() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='wrapper'><div id='button'>Button</div></div><script>document.getElementById('button').autofocus;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-form-control autofocus access should fail explicitly");

    assert!(session.to_string().contains("autofocus"));
}

#[test]
fn session_rejects_non_form_control_autocomplete_access_explicitly() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='wrapper'><button id='button'>Button</button></div><script>document.getElementById('button').autocomplete = 'on';</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-form-control autocomplete access should fail explicitly");

    assert!(session.to_string().contains("autocomplete"));
}

#[test]
fn session_resolves_input_file_accept_and_multiple_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><input id='upload' type='file'><div id='out'></div><script>const upload = document.getElementById('upload'); const before = String(upload.accept) + '|' + String(upload.multiple) + '|' + String(upload.hasAttribute('accept')) + '|' + String(upload.hasAttribute('multiple')); upload.accept = 'image/*'; upload.multiple = true; const afterSet = String(upload.accept) + '|' + String(upload.multiple) + '|' + String(upload.getAttribute('accept')) + '|' + String(upload.hasAttribute('multiple')); upload.accept = ''; upload.multiple = false; const afterClear = String(upload.accept) + '|' + String(upload.multiple) + '|' + String(upload.getAttribute('accept')) + '|' + String(upload.hasAttribute('multiple')); document.getElementById('out').textContent = before + ';' + afterSet + ';' + afterClear;</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute file input accept/multiple scripts");

    let upload_id = session.dom().select("#upload").unwrap()[0];
    let out_id = session.dom().select("#out").unwrap()[0];

    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "|false|false|false;image/*|true|image/*|true;|false||false"
    );
    assert_eq!(session.dom().select("#upload[accept]").unwrap().len(), 1);
    assert_eq!(session.dom().value_for_node(upload_id), "");
}

#[test]
fn session_rejects_non_input_accept_access_explicitly() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='wrapper'><button id='button'>Button</button></div><script>document.getElementById('button').accept = 'image/*';</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-input accept access should fail explicitly");

    assert!(session.to_string().contains("accept"));
}

#[test]
fn session_bootstraps_checkbox_indeterminate_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<input id='agree' type='checkbox'><div id='out'></div><script>const agree = document.getElementById('agree'); const before = agree.indeterminate; agree.indeterminate = true; document.getElementById('out').textContent = String(before) + ':' + String(agree.indeterminate) + ':' + String(document.querySelectorAll(':indeterminate').length);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute checkbox indeterminate scripts");

    let agree_id = session.dom().select("#agree").unwrap()[0];
    let out_id = session.dom().select("#out").unwrap()[0];

    assert_eq!(session.dom().indeterminate_for_node(agree_id), Some(true));
    assert_eq!(session.dom().text_content_for_node(out_id), "false:true:1");
}

#[test]
fn session_bootstraps_checkbox_default_checked_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<input id='agree' type='checkbox'><div id='out'></div><script>const agree = document.getElementById('agree'); const before = agree.defaultChecked; agree.defaultChecked = true; document.getElementById('out').textContent = String(before) + ':' + String(agree.defaultChecked) + ':' + String(agree.checked) + ':' + String(document.querySelectorAll(':default').length);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute checkbox defaultChecked scripts");

    let agree_id = session.dom().select("#agree").unwrap()[0];
    let out_id = session.dom().select("#out").unwrap()[0];

    assert_eq!(session.dom().checked_for_node(agree_id), Some(true));
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "false:true:true:1"
    );
    assert_eq!(session.dom().select(":default").unwrap().len(), 1);
}

#[test]
fn session_rejects_non_checkbox_default_checked_access_explicitly() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='wrapper'><textarea id='textarea'></textarea></div><script>document.getElementById('textarea').defaultChecked = true;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-checkbox defaultChecked access should fail explicitly");

    assert!(session.to_string().contains("defaultChecked"));
}

#[test]
fn session_rejects_non_form_control_length_access_explicitly() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='wrapper'><button id='button'>Button</button></div><script>document.getElementById('button').maxLength = 1;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-form-control maxLength access should fail explicitly");

    assert!(session.to_string().contains("maxLength"));
}

#[test]
fn session_click_clears_checkbox_indeterminate_and_dispatches_change() {
    let config = SessionConfig {
        url: "https://app.local/".to_string(),
        html: Some(
            "<input id='agree' type='checkbox'><div id='out'></div><script>const agree = document.getElementById('agree'); agree.indeterminate = true; agree.addEventListener('change', () => { const current = document.getElementById('agree'); document.getElementById('out').textContent = String(current.indeterminate) + ':' + String(current.checked); });</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    };

    let mut session = Session::new(config).expect("session should register listeners");
    let agree_id = session.dom().select("#agree").unwrap()[0];
    let out_id = session.dom().select("#out").unwrap()[0];

    session
        .click_node(agree_id)
        .expect("click should toggle checkbox and clear indeterminate");

    assert_eq!(session.dom().checked_for_node(agree_id), Some(true));
    assert_eq!(session.dom().indeterminate_for_node(agree_id), Some(false));
    assert_eq!(session.dom().text_content_for_node(out_id), "false:true");
}

#[test]
fn session_rejects_non_form_control_placeholder_access_explicitly() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='wrapper'><button id='button'>Button</button></div><script>document.getElementById('button').placeholder = 'hint';</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-form-control placeholder access should fail explicitly");

    assert!(session.to_string().contains("placeholder"));
}

#[test]
fn session_rejects_non_option_label_access_explicitly() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='wrapper'><button id='button'>Button</button></div><script>document.getElementById('button').label;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-option label access should fail explicitly");

    assert!(session.to_string().contains("label"));
}

#[test]
fn session_resolves_optgroup_disabled_and_label_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<select id='mode'><optgroup id='group' label='Cold'><option id='first' value='a'>A</option></optgroup></select><div id='out'></div><script>const group = document.getElementById('group'); const beforeDisabled = group.disabled; const beforeLabel = group.label; group.disabled = true; group.label = 'Warm'; document.getElementById('out').textContent = String(beforeDisabled) + ':' + beforeLabel + ':' + String(group.disabled) + ':' + String(group.getAttribute('disabled')) + ':' + group.label + ':' + group.getAttribute('label') + ':' + String(group.matches(':disabled')) + ':' + String(document.querySelectorAll(':disabled').length);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute optgroup reflection scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "false:Cold:true::Warm:Warm:true:1"
    );
}

#[test]
fn session_resolves_fieldset_disabled_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<fieldset id='group'></fieldset><div id='out'></div><script>const group = document.getElementById('group'); const before = group.disabled; group.disabled = true; document.getElementById('out').textContent = String(before) + ':' + String(group.disabled) + ':' + String(group.getAttribute('disabled')) + ':' + String(group.matches(':disabled')) + ':' + String(document.querySelectorAll(':disabled').length);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute fieldset reflection scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "false:true::true:1"
    );
}

#[test]
fn session_rejects_non_option_text_access_explicitly() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='wrapper'><button id='button'>Button</button></div><script>document.getElementById('button').text;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-option text access should fail explicitly");

    assert!(session.to_string().contains("text"));
}

#[test]
fn session_rejects_non_form_associated_form_access_explicitly() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='wrapper'><div id='box'></div></div><script>document.getElementById('box').form;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-form-associated form access should fail explicitly");

    assert!(session.to_string().contains("form"));
}

#[test]
fn session_rejects_non_form_control_disabled_access_getter_explicitly() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='wrapper'><div id='box'></div></div><script>document.getElementById('box').disabled;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-form-control disabled access should fail explicitly");

    assert!(session.to_string().contains("disabled"));
}

#[test]
fn session_rejects_non_form_control_disabled_access_explicitly() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='wrapper'><div id='box'></div></div><script>document.getElementById('box').disabled = true;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-form-control disabled access should fail explicitly");

    assert!(session.to_string().contains("disabled"));
}

#[test]
fn session_rejects_non_select_selected_index_access_explicitly() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='wrapper'><button id='button'>Button</button></div><script>document.getElementById('button').selectedIndex;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-select selectedIndex access should fail explicitly");

    assert!(session.to_string().contains("selectedIndex"));
}

#[test]
fn session_resolves_input_and_button_type_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><input id='field'><button id='action'></button></div><div id='out'></div><script>const input = document.getElementById('field'); const button = document.getElementById('action'); const before = input.type + '|' + button.type; input.type = 'email'; button.type = 'reset'; const afterSet = input.type + '|' + button.type + '|' + input.getAttribute('type') + '|' + button.getAttribute('type'); input.type = 'bogus'; button.type = 'bogus'; const afterInvalid = input.type + '|' + button.type + '|' + input.getAttribute('type') + '|' + button.getAttribute('type'); document.getElementById('out').textContent = before + ';' + afterSet + ';' + afterInvalid;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute input.type and button.type scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "text|submit;email|reset|email|reset;text|submit|bogus|bogus"
    );
}

#[test]
fn session_rejects_non_form_control_type_access_explicitly() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='wrapper'><div id='box'></div></div><script>document.getElementById('box').type;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-form-control type access should fail explicitly");

    assert!(session.to_string().contains("type"));
}

#[test]
fn session_rejects_non_option_index_access_explicitly() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='wrapper'><button id='button'>Button</button></div><script>document.getElementById('button').index;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-option index access should fail explicitly");

    assert!(session.to_string().contains("index"));
}

#[test]
fn session_rejects_non_option_default_selected_access_explicitly() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='wrapper'><button id='button'>Button</button></div><script>document.getElementById('button').defaultSelected;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-option defaultSelected access should fail explicitly");

    assert!(session.to_string().contains("defaultSelected"));
}

#[test]
fn session_resolves_element_labels_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><label id='explicit-label' for='control'>Explicit</label><input id='control' value='A'><label id='implicit-label'><input id='inner-control' value='B'>Implicit</label><div id='wrapper'></div></div><div id='out'></div><script>const control = document.getElementById('control'); const labels = control.labels; const inner = document.getElementById('inner-control').labels; const before = labels.length; document.getElementById('wrapper').innerHTML = '<label id=\"second-label\" for=\"control\">Second</label>'; document.getElementById('out').textContent = String(before) + ':' + String(labels.length) + ':' + labels.item(0).getAttribute('id') + ':' + labels.item(1).textContent + ':' + String(inner.length) + ':' + inner.item(0).getAttribute('id');</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute labels scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "1:2:explicit-label:Second:1:implicit-label"
    );
}

#[test]
fn session_resolves_fieldset_elements_and_datalist_options_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><fieldset id='fieldset'><input name='first' value='Ada'><textarea name='bio'>Bio</textarea></fieldset><datalist id='list'><option name='alpha' value='a'>A</option><option id='second' value='b'>B</option></datalist></div><div id='out'></div><script>const elements = document.getElementById('fieldset').elements; const options = document.getElementById('list').options; const beforeElements = elements.length; const beforeOptions = options.length; const first = elements.item(0); const namedElement = elements.namedItem('first'); const namedOption = options.namedItem('second'); document.getElementById('fieldset').textContent = 'gone'; document.getElementById('list').textContent = 'gone'; document.getElementById('out').textContent = String(beforeElements) + ':' + String(elements.length) + ':' + String(beforeOptions) + ':' + String(options.length) + ':' + first.value + ':' + namedElement.value + ':' + namedOption.textContent + ':' + String(options.namedItem('missing'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute fieldset.elements and datalist.options scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "2:0:2:0:Ada:Ada:B:null"
    );
}

#[test]
fn session_resolves_fieldset_elements_and_datalist_options_iterator_helpers_through_inline_scripts()
{
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><fieldset id='fieldset'><input id='first-control' name='first' value='Ada'><textarea id='second-control' name='bio'>Bio</textarea></fieldset><datalist id='list'><option id='first-option' name='alpha' value='a'>A</option><option id='second-option' value='b'>B</option></datalist></div><div id='out'></div><script>const elements = document.getElementById('fieldset').elements; const options = document.getElementById('list').options; const elementKeys = elements.keys(); const elementValues = elements.values(); const elementEntries = elements.entries(); const optionKeys = options.keys(); const optionValues = options.values(); const optionEntries = options.entries(); const firstElementKey = elementKeys.next(); const firstElementValue = elementValues.next(); const firstElementEntry = elementEntries.next(); const firstOptionKey = optionKeys.next(); const firstOptionValue = optionValues.next(); const firstOptionEntry = optionEntries.next(); let serial = ''; elements.forEach((element, index, list) => { serial += 'E' + String(index) + ':' + element.getAttribute('id') + ':' + String(list.length) + ';'; }); options.forEach((element, index, list) => { serial += 'O' + String(index) + ':' + element.getAttribute('id') + ':' + String(list.length) + ';'; }); document.getElementById('out').textContent = String(firstElementKey.value) + ':' + firstElementValue.value.getAttribute('id') + ':' + String(firstElementEntry.value.index) + ':' + firstElementEntry.value.value.getAttribute('id') + ':' + String(firstOptionKey.value) + ':' + firstOptionValue.value.getAttribute('id') + ':' + String(firstOptionEntry.value.index) + ':' + firstOptionEntry.value.value.getAttribute('id') + ':' + serial;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute fieldset.elements and datalist.options iterator helper scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "0:first-control:0:first-control:0:first-option:0:first-option:E0:first-control:2;E1:second-control:2;O0:first-option:2;O1:second-option:2;"
    );
}

#[test]
fn session_resolves_map_areas_and_table_t_bodies_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><map id='map'><area id='first-area' name='first' href='/first'><area id='second-area' name='second' href='/second'></map><table id='table'><tbody id='first-body'><tr><td>One</td></tr></tbody></table></div><div id='out'></div><script>const areas = document.getElementById('map').areas; const bodies = document.getElementById('table').tBodies; const beforeAreas = areas.length; const beforeBodies = bodies.length; const firstArea = areas.item(0); const firstBody = bodies.item(0); document.getElementById('map').innerHTML += '<area id=\"third-area\" name=\"third\" href=\"/third\">'; document.getElementById('table').innerHTML += '<tbody id=\"second-body\"></tbody>'; document.getElementById('out').textContent = String(beforeAreas) + ':' + String(areas.length) + ':' + String(beforeBodies) + ':' + String(bodies.length) + ':' + String(firstArea.getAttribute('id')) + ':' + String(firstBody.getAttribute('id')) + ':' + String(areas.namedItem('third-area')) + ':' + String(bodies.namedItem('second-body')) + ':' + String(areas.namedItem('missing'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute map.areas and table.tBodies scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "2:3:1:2:first-area:first-body:[object Element]:[object Element]:null"
    );
}

#[test]
fn session_resolves_map_areas_and_table_t_bodies_iterator_helpers_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><map id='map'><area id='first-area' name='first' href='/first'><area id='second-area' name='second' href='/second'></map><table id='table'><tbody id='first-body'><tr><td>One</td></tr></tbody></table></div><div id='out'></div><script>const areas = document.getElementById('map').areas; const bodies = document.getElementById('table').tBodies; const areaKeys = areas.keys(); const areaValues = areas.values(); const areaEntries = areas.entries(); const bodyKeys = bodies.keys(); const bodyValues = bodies.values(); const bodyEntries = bodies.entries(); const firstAreaKey = areaKeys.next(); const firstAreaValue = areaValues.next(); const firstAreaEntry = areaEntries.next(); const firstBodyKey = bodyKeys.next(); const firstBodyValue = bodyValues.next(); const firstBodyEntry = bodyEntries.next(); let serial = ''; areas.forEach((element, index, list) => { serial += 'A' + String(index) + ':' + element.getAttribute('id') + ':' + String(list.length) + ';'; }); bodies.forEach((element, index, list) => { serial += 'B' + String(index) + ':' + element.getAttribute('id') + ':' + String(list.length) + ';'; }); document.getElementById('out').textContent = String(firstAreaKey.value) + ':' + firstAreaValue.value.getAttribute('id') + ':' + String(firstAreaEntry.value.index) + ':' + firstAreaEntry.value.value.getAttribute('id') + ':' + String(firstBodyKey.value) + ':' + firstBodyValue.value.getAttribute('id') + ':' + String(firstBodyEntry.value.index) + ':' + firstBodyEntry.value.value.getAttribute('id') + ':' + serial;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute map.areas and table.tBodies iterator helper scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "0:first-area:0:first-area:0:first-body:0:first-body:A0:first-area:2;A1:second-area:2;B0:first-body:1;"
    );
}

#[test]
fn session_resolves_document_images_and_links_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><img id='hero' name='hero' alt='Hero'><img name='thumb' alt='Thumb'><a id='docs' href='/docs'>Docs</a><a id='plain'>Plain</a><area id='map' name='map' href='/map'></div><div id='out'></div><script>const images = document.images; const links = document.links; const beforeImages = images.length; const beforeLinks = links.length; const hero = images.namedItem('hero'); const thumb = images.namedItem('thumb'); const docs = links.namedItem('docs'); const map = links.namedItem('map'); document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(beforeImages) + ':' + String(images.length) + ':' + String(beforeLinks) + ':' + String(links.length) + ':' + String(hero) + ':' + String(thumb) + ':' + String(docs) + ':' + String(map) + ':' + String(links.namedItem('plain'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute document.images and document.links scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "2:0:2:0:[object Element]:[object Element]:[object Element]:[object Element]:null"
    );
}

#[test]
fn session_resolves_document_links_iterator_helpers_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><img id='hero' name='hero' alt='Hero'><img name='thumb' alt='Thumb'><a id='docs' href='/docs'>Docs</a><a id='plain'>Plain</a><area id='map' name='map' href='/map'></div><div id='out'></div><script>const links = document.links; const keys = links.keys(); const values = links.values(); const entries = links.entries(); const firstKey = keys.next(); const firstValue = values.next(); const firstEntry = entries.next(); let out = ''; links.forEach((element, index, list) => { out += String(index) + ':' + element.getAttribute('id') + ':' + String(list.length) + ';'; }); document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(firstKey.value) + ':' + firstValue.value.getAttribute('id') + ':' + String(firstEntry.value.index) + ':' + firstEntry.value.value.getAttribute('id') + ':' + out;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute document.links iterator helper scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "0:docs:0:docs:0:docs:2;1:map:2;"
    );
}

#[test]
fn session_resolves_document_images_iterator_helpers_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><img id='hero' name='hero' alt='Hero'><img id='thumb' name='thumb' alt='Thumb'></div><div id='out'></div><script>const images = document.images; const keys = images.keys(); const values = images.values(); const entries = images.entries(); const firstKey = keys.next(); const firstValue = values.next(); const firstEntry = entries.next(); let out = ''; images.forEach((element, index, list) => { out += String(index) + ':' + element.getAttribute('id') + ':' + String(list.length) + ';'; }); document.getElementById('out').textContent = String(firstKey.value) + ':' + firstValue.value.getAttribute('id') + ':' + String(firstEntry.value.index) + ':' + firstEntry.value.value.getAttribute('id') + ':' + out;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute document.images iterator helper scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "0:hero:0:hero:0:hero:2;1:thumb:2;"
    );
}

#[test]
fn session_resolves_document_anchors_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><a name='first'>First</a><a id='ignored'>Ignored</a></div><div id='out'></div><script>const anchors = document.anchors; const before = anchors.length; const first = anchors.namedItem('first'); const root = document.getElementById('root'); root.innerHTML = root.innerHTML + '<a name=\"second\">Second</a>'; document.getElementById('out').textContent = String(before) + ':' + String(anchors.length) + ':' + first.textContent + ':' + anchors.namedItem('second').textContent + ':' + String(anchors.namedItem('missing'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute document.anchors scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "1:2:First:Second:null"
    );
}

#[test]
fn session_resolves_document_children_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><span>First</span></main><div id='out'></div><script>const children = document.children; const before = children.length; const first = children.item(0); const root = children.namedItem('root'); document.getElementById('root').remove(); document.getElementById('out').textContent = String(before) + ':' + String(children.length) + ':' + String(first) + ':' + String(root) + ':' + String(children.namedItem('root'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute document.children scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "3:2:[object Element]:[object Element]:null"
    );
    assert_eq!(session.dom().select("#root").unwrap().len(), 0);
}

#[test]
fn session_resolves_child_nodes_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<!--pre--><main id='root'>Hello<span>World</span><!--tail--></main><div id='out'></div><script>const docNodes = document.childNodes; const rootNodes = document.getElementById('root').childNodes; const docFirst = docNodes.item(0); const docSecond = docNodes.item(1); const rootValues = rootNodes.values(); const firstRoot = rootValues.next(); const secondRoot = rootValues.next(); const thirdRoot = rootValues.next(); document.getElementById('out').textContent = String(docNodes.length) + ':' + docFirst.nodeName + ':' + String(docFirst.nodeType) + ':' + String(docFirst) + ':' + docSecond.nodeName + ':' + String(docSecond.nodeType) + ':' + firstRoot.value.nodeName + ':' + String(firstRoot.value.nodeType) + ':' + firstRoot.value.textContent + ':' + secondRoot.value.nodeName + ':' + String(secondRoot.value.nodeType) + ':' + secondRoot.value.textContent + ':' + thirdRoot.value.nodeName + ':' + String(thirdRoot.value.nodeType) + ':' + thirdRoot.value.textContent;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute childNodes scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "4:#comment:8:[object Node]:main:1:#text:3:Hello:span:1:World:#comment:8:"
    );
}

#[test]
fn session_resolves_template_content_live_collections_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some("<template id='tpl'><span id='inner'>Inner</span></template><div id='out'></div><script>const tpl = document.getElementById('tpl'); const content = tpl.content; const nodes = content.childNodes; const children = content.children; const before = nodes.length; tpl.innerHTML += '<!--tail--><span id=\"second\">Second</span>'; document.getElementById('out').textContent = String(content) + ':' + String(before) + ':' + String(nodes.length) + ':' + nodes.item(1).nodeName + ':' + String(children.length) + ':' + String(children.namedItem('second').textContent);</script>".to_string()),
        local_storage: BTreeMap::new(),
    })
    .expect("template content markup should parse");

    let out_id = session
        .dom()
        .indexes()
        .id_index
        .get("out")
        .copied()
        .expect("out element should exist");
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "[object DocumentFragment]:1:3:#comment:2:Second"
    );
}

#[test]
fn session_resolves_template_content_query_selector_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some("<template id='tpl'><span class='primary' id='first'>First</span><span class='primary' id='second'>Second</span></template><div id='out'></div><script>const tpl = document.getElementById('tpl'); const content = tpl.content; const first = content.querySelector('.primary'); const all = content.querySelectorAll('.primary'); document.getElementById('out').textContent = String(content) + ':' + first.textContent + ':' + String(all.length) + ':' + all.item(0).textContent + ':' + all.item(1).textContent + ':' + String(all.item(2));</script>".to_string()),
        local_storage: BTreeMap::new(),
    })
    .expect("template content query selectors should parse");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "[object DocumentFragment]:First:2:First:Second:null"
    );
}

#[test]
fn session_resolves_template_content_get_element_by_id_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some("<template id='tpl'><span id='foo,bar'>First</span></template><div id='out'></div><script>const tpl = document.getElementById('tpl'); const content = tpl.content; const hit = content.getElementById('foo,bar'); document.getElementById('out').textContent = String(hit) + ':' + hit.textContent;</script>".to_string()),
        local_storage: BTreeMap::new(),
    })
    .expect("template content getElementById should parse");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "[object Element]:First"
    );
}

#[test]
fn session_resolves_template_content_inner_html_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some("<template id='tpl'><span id='inner'>Inner</span></template><div id='out'></div><script>const tpl = document.getElementById('tpl'); const content = tpl.content; const before = content.innerHTML; content.innerHTML = '<!--tail--><span id=\"second\">Second</span>'; document.getElementById('out').textContent = before + '|' + content.innerHTML + '|' + String(content.childNodes.length) + ':' + content.childNodes.item(0).nodeName + ':' + String(content.children.length) + ':' + content.children.namedItem('second').textContent;</script>".to_string()),
        local_storage: BTreeMap::new(),
    })
    .expect("template content innerHTML should parse");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "<span id=\"inner\">Inner</span>|<!--tail--><span id=\"second\">Second</span>|2:#comment:1:Second"
    );
    assert_eq!(session.dom().select("#second").unwrap().len(), 1);
}

#[test]
fn session_resolves_document_create_document_fragment_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'></main><div id='out'></div><script>const root = document.getElementById('root'); const frag = document.createDocumentFragment(); frag.appendChild(document.createTextNode('Hello')); const returned = root.appendChild(frag); document.getElementById('out').textContent = String(returned) + ':' + String(frag.childNodes.length) + ':' + root.textContent + ':' + String(root.childNodes.length);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("document.createDocumentFragment should parse");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "[object DocumentFragment]:0:Hello:1"
    );
}

#[test]
fn session_resolves_document_import_node_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'></main><div id='out'></div><script>const root = document.getElementById('root'); const source = document.createDocumentFragment(); source.appendChild(document.createTextNode('Hello')); const imported = document.importNode(source, true); const returned = root.appendChild(imported); document.getElementById('out').textContent = String(returned) + ':' + String(imported.childNodes.length) + ':' + root.textContent + ':' + String(root.childNodes.length);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("document.importNode should parse");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "[object DocumentFragment]:0:Hello:1"
    );
}

#[test]
fn session_resolves_node_normalize_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'>First</main><div id='out'></div><script>const root = document.getElementById('root'); root.appendChild(document.createTextNode('Second')); root.appendChild(document.createTextNode('Third')); root.normalize(); document.getElementById('out').textContent = String(root.childNodes.length) + ':' + String(root.firstChild.nodeType) + ':' + root.textContent;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("Node.normalize should parse");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "1:3:FirstSecondThird"
    );
}

#[test]
fn session_reorders_text_nodes_with_node_before_and_after_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'></main><div id='out'></div><script>const root = document.getElementById('root'); const child = document.createTextNode('Hello'); root.appendChild(child); child.before(document.createTextNode('Before')); child.after(document.createTextNode('After')); document.getElementById('out').textContent = String(root.childNodes.length) + ':' + root.textContent;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("Node.before and Node.after should parse");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "3:BeforeHelloAfter"
    );
}

#[test]
fn session_resolves_node_remove_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'></main><div id='out'></div><script>const root = document.getElementById('root'); const child = document.createTextNode('Hello'); root.appendChild(child); child.remove(); document.getElementById('out').textContent = String(child.parentNode) + ':' + String(root.childNodes.length);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("Node.remove should parse");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "null:0");
}

#[test]
fn session_resolves_element_remove_child_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><span id='first'>First</span><span id='second'>Second</span></main><div id='out'></div><script>const out = document.getElementById('out'); const root = document.getElementById('root'); const child = document.getElementById('first'); out.textContent = String(root.removeChild(child)) + ':' + String(root.children.length) + ':' + String(child.parentNode);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("Element.removeChild should parse");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "[object Element]:1:null"
    );
}

#[test]
fn session_serializes_namespace_aware_names_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some("<main id='root'><svg id='icon' viewbox='0 0 10 10'><foreignobject id='foreign'><div id='html'>Text</div></foreignobject></svg><math id='formula' definitionurl='https://example.com'><mi id='symbol'>x</mi></math><div id='out'></div><script>const icon = document.getElementById('icon'); const formula = document.getElementById('formula'); document.getElementById('out').textContent = icon.outerHTML + '|' + formula.outerHTML;</script></main>".to_string()),
        local_storage: BTreeMap::new(),
    })
    .expect("namespace-aware serialization should parse");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "<svg id=\"icon\" viewBox=\"0 0 10 10\"><foreignObject id=\"foreign\"><div id=\"html\">Text</div></foreignObject></svg>|<math definitionURL=\"https://example.com\" id=\"formula\"><mi id=\"symbol\">x</mi></math>"
    );
}

#[test]
fn session_resolves_table_rows_and_row_cells_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<table id='table'><thead id='head'><tr id='head-row'><th id='head-cell'>H</th></tr></thead><tbody id='body'><tr id='first-row'><td id='first-cell'>A</td></tr></tbody><tfoot id='foot'><tr id='foot-row'><td id='foot-cell'>F</td></tr></tfoot></table><div id='out'></div><script>const table = document.getElementById('table'); const body = document.getElementById('body'); const row = document.getElementById('first-row'); const rows = table.rows; const bodyRows = body.rows; const cells = row.cells; const before = String(rows.length) + ':' + String(bodyRows.length) + ':' + String(cells.length) + ':' + String(rows.namedItem('first-row')) + ':' + String(cells.namedItem('first-cell')); body.innerHTML = body.innerHTML + '<tr id=\"second-row\"><td id=\"second-cell\">B</td><td id=\"third-cell\">C</td></tr>'; row.append(document.getElementById('third-cell')); document.getElementById('out').textContent = before + '|' + String(rows.length) + ':' + String(bodyRows.length) + ':' + String(cells.length) + ':' + String(rows.namedItem('second-row')) + ':' + String(bodyRows.namedItem('second-row')) + ':' + String(cells.namedItem('third-cell'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute table.rows and tr.cells scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "3:1:1:[object Element]:[object Element]|4:2:2:[object Element]:[object Element]:[object Element]"
    );
    assert_eq!(session.dom().select("#second-row").unwrap().len(), 1);
}

#[test]
fn session_resolves_table_section_rows_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<table id='table'><thead id='head'><tr id='head-row'><th id='head-cell'>H</th></tr></thead><tbody id='body'><tr id='first-row'><td id='first-cell'>A</td></tr></tbody><tfoot id='foot'><tr id='foot-row'><td id='foot-cell'>F</td></tr></tfoot></table><div id='out'></div><script>const head = document.getElementById('head'); const foot = document.getElementById('foot'); document.getElementById('out').textContent = String(head.rows.length) + ':' + String(head.rows.item(0)) + ':' + String(foot.rows.length) + ':' + String(foot.rows.item(0));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute table section rows scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "1:[object Element]:1:[object Element]"
    );
}

#[test]
fn session_resolves_table_section_rows_named_item_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<table id='table'><thead id='head'><tr id='head-row'><th id='head-cell'>H</th></tr></thead><tbody id='body'><tr id='first-row'><td id='first-cell'>A</td></tr></tbody><tfoot id='foot'><tr id='foot-row'><td id='foot-cell'>F</td></tr></tfoot></table><div id='out'></div><script>const head = document.getElementById('head'); const foot = document.getElementById('foot'); document.getElementById('out').textContent = String(head.rows.namedItem('head-row')) + ':' + String(head.rows.namedItem('missing')) + ':' + String(foot.rows.namedItem('foot-row')) + ':' + String(foot.rows.namedItem('missing'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute table section rows namedItem scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "[object Element]:null:[object Element]:null"
    );
}

#[test]
fn session_resolves_table_section_rows_iterator_helpers_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<table id='table'><thead id='head'><tr id='head-row'><th id='head-cell'>H</th></tr></thead><tbody id='body'><tr id='body-row'><td id='body-cell'>B</td></tr></tbody><tfoot id='foot'><tr id='foot-row'><td id='foot-cell'>F</td></tr></tfoot></table><div id='out'></div><script>const head = document.getElementById('head'); const body = document.getElementById('body'); const foot = document.getElementById('foot'); const bodyRows = body.rows; const bodyKeys = bodyRows.keys(); const bodyValues = bodyRows.values(); const bodyEntries = bodyRows.entries(); const firstKey = bodyKeys.next(); const firstValue = bodyValues.next(); const firstEntry = bodyEntries.next(); let serial = ''; bodyRows.forEach((element, index, list) => { serial += 'B' + String(index) + ':' + element.getAttribute('id') + ':' + String(list.length) + ';'; }); document.getElementById('out').textContent = String(head.rows.length) + ':' + String(bodyRows.length) + ':' + String(foot.rows.length) + ':' + String(firstKey.value) + ':' + firstValue.value.getAttribute('id') + ':' + String(firstEntry.value.index) + ':' + firstEntry.value.value.getAttribute('id') + ':' + serial;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute table section rows iterator helper scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "1:1:1:0:body-row:0:body-row:B0:body-row:1;"
    );
}

#[test]
fn session_resolves_table_rows_and_row_cells_iterator_helpers_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<table id='table'><thead id='head'><tr id='head-row'><th id='head-cell'>H</th></tr></thead><tbody id='body'><tr id='first-row'><td id='first-cell'>A</td></tr></tbody><tfoot id='foot'><tr id='foot-row'><td id='foot-cell'>F</td></tr></tfoot></table><div id='out'></div><script>const table = document.getElementById('table'); const row = document.getElementById('first-row'); const tableKeys = table.rows.keys(); const tableValues = table.rows.values(); const tableEntries = table.rows.entries(); const rowKeys = row.cells.keys(); const rowValues = row.cells.values(); const rowEntries = row.cells.entries(); const firstTableKey = tableKeys.next(); const firstTableValue = tableValues.next(); const firstTableEntry = tableEntries.next(); const firstRowKey = rowKeys.next(); const firstRowValue = rowValues.next(); const firstRowEntry = rowEntries.next(); let out = ''; table.rows.forEach((element, index, list) => { out += 'T' + String(index) + ':' + element.getAttribute('id') + ':' + String(list.length) + ';'; }); row.cells.forEach((element, index, list) => { out += 'R' + String(index) + ':' + element.getAttribute('id') + ':' + String(list.length) + ';'; }); document.getElementById('out').textContent = String(firstTableKey.value) + ':' + firstTableValue.value.getAttribute('id') + ':' + String(firstTableEntry.value.index) + ':' + firstTableEntry.value.value.getAttribute('id') + ':' + String(firstRowKey.value) + ':' + firstRowValue.value.getAttribute('id') + ':' + String(firstRowEntry.value.index) + ':' + firstRowEntry.value.value.getAttribute('id') + ':' + out;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute table.rows and row.cells iterator helper scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "0:head-row:0:head-row:0:first-cell:0:first-cell:T0:head-row:3;T1:first-row:3;T2:foot-row:3;R0:first-cell:1;"
    );
}

#[test]
fn session_resolves_document_scripts_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><script id='first-script'></script></div><div id='out'></div><script>const out = document.getElementById('out'); const scripts = document.scripts; const before = scripts.length; const first = scripts.namedItem('first-script'); document.getElementById('root').textContent = 'gone'; out.textContent = String(before) + ':' + String(scripts.length) + ':' + String(first) + ':' + String(scripts.namedItem('missing'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute document.scripts scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "2:1:[object Element]:null"
    );
}

#[test]
fn session_resolves_document_scripts_iterator_helpers_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><script id='first-script'></script></div><div id='out'></div><script id='current-script'>const out = document.getElementById('out'); const scripts = document.scripts; const keys = scripts.keys(); const values = scripts.values(); const entries = scripts.entries(); const firstKey = keys.next(); const firstValue = values.next(); const firstEntry = entries.next(); let serial = ''; scripts.forEach((element, index, list) => { serial += String(index) + ':' + element.getAttribute('id') + ':' + String(list.length) + ';'; }); document.getElementById('root').textContent = 'gone'; out.textContent = String(firstKey.value) + ':' + firstValue.value.getAttribute('id') + ':' + String(firstEntry.value.index) + ':' + firstEntry.value.value.getAttribute('id') + ':' + serial + ':' + String(scripts.length);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute document.scripts iterator helper scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "0:first-script:0:first-script:0:first-script:2;1:current-script:2;:1"
    );
}

#[test]
fn session_resolves_document_active_element_through_inline_scripts() {
    let mut session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<input id='first'><div id='out'></div><script>document.getElementById('first').addEventListener('focus', () => { document.getElementById('out').textContent = document.activeElement.getAttribute('id'); });</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute document.activeElement scripts");

    let first_id = session.dom().select("#first").unwrap()[0];
    let out_id = session.dom().select("#out").unwrap()[0];
    session.focus_node(first_id).expect("focus should work");

    assert_eq!(session.dom().text_content_for_node(out_id), "first");
}

#[test]
fn session_document_has_focus_tracks_focus_state() {
    let mut session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some("<input id='first'><div id='out'></div>".to_string()),
        local_storage: BTreeMap::new(),
    })
    .expect("session should build");

    let first_id = session.dom().select("#first").unwrap()[0];
    assert!(!session.document_has_focus());

    session.focus_node(first_id).expect("focus should work");
    assert!(session.document_has_focus());

    session.blur_node(first_id).expect("blur should work");
    assert!(!session.document_has_focus());
}

#[test]
fn session_element_click_focus_and_blur_work_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<input id='agree' type='checkbox'><input id='first'><div id='out'></div><script>const checkbox = document.getElementById('agree'); const first = document.getElementById('first'); checkbox.click(); first.focus(); const focused = document.activeElement.getAttribute('id'); first.blur(); document.getElementById('out').textContent = String(checkbox.checked) + ':' + String(focused);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute Element.click/focus/blur scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "true:first");
}

#[test]
fn session_resolves_document_style_sheets_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><style id='first-style'>.primary { color: red; }</style><link id='first-link' rel='stylesheet' href='a.css'><link id='ignored-link' rel='preload' href='b.css'></div><div id='out'></div><script>const sheets = document.styleSheets; const before = sheets.length; const first = sheets.item(0); const second = sheets.item(1); document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(before) + ':' + String(sheets.length) + ':' + String(first) + ':' + String(second) + ':' + String(sheets.item(2));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute document.styleSheets scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "2:0:[object CSSStyleSheet]:[object CSSStyleSheet]:null"
    );
}

#[test]
fn session_resolves_document_style_sheets_named_item_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><style id='first-style'>.primary { color: red; }</style><link id='first-link' rel='stylesheet' href='a.css'><link id='ignored-link' rel='preload' href='b.css'></div><div id='out'></div><script>const sheets = document.styleSheets; const first = sheets.namedItem('first-style'); const second = sheets.namedItem('first-link'); document.getElementById('out').textContent = String(sheets.length) + ':' + String(first) + ':' + String(second) + ':' + String(sheets.namedItem('missing'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute document.styleSheets namedItem scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "2:[object CSSStyleSheet]:[object CSSStyleSheet]:null"
    );
}

#[test]
fn session_resolves_document_style_sheets_for_each_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><style id='first-style'>.primary { color: red; }</style><link id='first-link' rel='stylesheet' href='a.css'></div><div id='out'></div><script>const sheets = document.styleSheets; let out = ''; sheets.forEach((sheet, index, list) => { out += String(index) + ':' + String(sheet) + ':' + String(list.length) + ';'; }); document.getElementById('out').textContent = out;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute document.styleSheets.forEach scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "0:[object CSSStyleSheet]:2;1:[object CSSStyleSheet]:2;"
    );
}

#[test]
fn session_reports_document_style_sheets_for_each_callback_errors_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>document.styleSheets.forEach(123);</script>".to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("document.styleSheets.forEach should require an arrow function callback");

    assert!(
        error
            .to_string()
            .contains("StyleSheetList.forEach() requires an arrow function callback")
    );
}

#[test]
fn session_reports_table_rows_on_non_table_elements_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='bad'></div><div id='out'></div><script>document.getElementById('bad').rows.length;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-table rows access should fail explicitly");

    assert!(error.to_string().contains("Script error"));
    assert!(error.to_string().contains("table.rows"));
    assert!(
        error
            .to_string()
            .contains("supported table.rows host element")
    );
}

#[test]
fn session_reports_map_areas_on_non_map_elements_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='bad'></div><div id='out'></div><script>document.getElementById('bad').areas.length;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-map areas access should fail explicitly");

    assert!(error.to_string().contains("Script error"));
    assert!(error.to_string().contains("map.areas"));
    assert!(
        error
            .to_string()
            .contains("supported map.areas host element")
    );
}

#[test]
fn session_reports_table_t_bodies_on_non_table_elements_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='bad'></div><div id='out'></div><script>document.getElementById('bad').tBodies.length;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-table tBodies access should fail explicitly");

    assert!(error.to_string().contains("Script error"));
    assert!(error.to_string().contains("table.tBodies"));
    assert!(
        error
            .to_string()
            .contains("supported table.tBodies host element")
    );
}

#[test]
fn session_resolves_document_applets_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><applet id='first-applet' name='first-applet'>First</applet><applet name='second-applet'>Second</applet></div><div id='out'></div><script>const applets = document.applets; const before = applets.length; const first = applets.namedItem('first-applet'); document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(before) + ':' + String(applets.length) + ':' + String(first) + ':' + String(applets.namedItem('missing'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute document.applets scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "2:0:[object Element]:null"
    );
}

#[test]
fn session_resolves_document_applets_iterator_helpers_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><applet id='first-applet' name='first-applet'>First</applet><applet id='second-applet' name='second-applet'>Second</applet></div><div id='out'></div><script>const applets = document.applets; const keys = applets.keys(); const values = applets.values(); const entries = applets.entries(); const firstKey = keys.next(); const firstValue = values.next(); const firstEntry = entries.next(); let out = ''; applets.forEach((element, index, list) => { out += String(index) + ':' + element.textContent + ':' + String(list.length) + ';'; }); document.getElementById('out').textContent = String(firstKey.value) + ':' + firstValue.value.textContent + ':' + String(firstEntry.value.index) + ':' + firstEntry.value.value.textContent + ':' + out;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute document.applets iterator helper scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "0:First:0:First:0:First:2;1:Second:2;"
    );
}

#[test]
fn session_resolves_has_child_nodes_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><span id='child'>Child</span></main><template id='tpl'><span id='inner'>Inner</span></template><div id='out'></div><script>const root = document.getElementById('root'); const child = document.getElementById('child'); const tpl = document.getElementById('tpl'); document.getElementById('out').textContent = String(document.hasChildNodes()) + ':' + String(root.hasChildNodes()) + ':' + String(child.hasChildNodes()) + ':' + String(tpl.content.hasChildNodes());</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute hasChildNodes scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "true:true:true:true"
    );
}

#[test]
fn session_resolves_first_and_last_child_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<!--pre--><html><head></head><body>Text<div id='out'></div><main id='root'><span id='child'>Child</span></main><template id='tpl'><span id='inner'>Inner</span><!--tail--></template><script>const html = document.documentElement; const body = document.body; const text = body.childNodes.item(0); const tpl = document.getElementById('tpl'); document.getElementById('out').textContent = String(document.firstChild) + ':' + String(document.lastChild) + ':' + String(html.firstChild) + ':' + String(html.lastChild) + ':' + String(body.firstChild) + ':' + String(body.lastChild) + ':' + String(text.firstChild) + ':' + String(text.lastChild) + ':' + String(tpl.content.firstChild) + ':' + String(tpl.content.lastChild);</script><!--body-tail--></body></html>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute firstChild/lastChild scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "[object Node]:[object Element]:[object Element]:[object Element]:[object Node]:[object Node]:null:null:[object Element]:[object Node]"
    );
}

#[test]
fn session_resolves_sibling_access_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<!--pre--><html><head></head><body>Text<div id='out'></div><main id='root'><span id='child'>Child</span></main><template id='tpl'><span id='inner'>Inner</span><!--tail--></template><script>const html = document.documentElement; const head = document.head; const body = document.body; const tpl = document.getElementById('tpl'); const content = tpl.content; const text = body.childNodes.item(0); const out = body.childNodes.item(1); document.getElementById('out').textContent = String(document.nextSibling) + ':' + String(document.previousSibling) + ':' + String(html.previousSibling) + ':' + String(head.nextSibling) + ':' + String(body.previousSibling) + ':' + String(body.nextSibling) + ':' + String(body.firstChild.nextSibling) + ':' + String(body.lastChild.previousSibling) + ':' + String(text.nextSibling) + ':' + String(out.previousSibling) + ':' + String(tpl.nextSibling) + ':' + String(tpl.previousSibling) + ':' + String(content.nextSibling) + ':' + String(content.previousSibling) + ':' + String(content.firstChild.nextSibling) + ':' + String(content.lastChild.previousSibling);</script></body></html>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute sibling access scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "null:null:[object Node]:[object Element]:[object Element]:null:[object Element]:[object Element]:[object Element]:[object Node]:[object Element]:[object Element]:null:null:[object Node]:[object Element]"
    );
}

#[test]
fn session_resolves_element_sibling_access_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<!--pre--><html><head></head><body>Text<div id='out'></div><main id='root'><span id='child'>Child</span></main><script>const html = document.documentElement; const head = document.head; const body = document.body; const text = body.firstChild; const out = body.childNodes.item(1); const main = body.childNodes.item(2); const script = body.lastChild; document.getElementById('out').textContent = String(document.nextElementSibling) + ':' + String(document.previousElementSibling) + ':' + String(html.nextElementSibling) + ':' + String(html.previousElementSibling) + ':' + String(head.nextElementSibling) + ':' + String(head.previousElementSibling) + ':' + String(body.nextElementSibling) + ':' + String(body.previousElementSibling) + ':' + String(text.nextElementSibling) + ':' + String(text.previousElementSibling) + ':' + String(out.nextElementSibling) + ':' + String(out.previousElementSibling) + ':' + String(main.previousElementSibling) + ':' + String(main.nextElementSibling) + ':' + String(script.previousElementSibling) + ':' + String(script.nextElementSibling);</script></body></html>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute element sibling access scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "null:null:null:null:[object Element]:null:null:[object Element]:[object Element]:null:[object Element]:null:[object Element]:[object Element]:[object Element]:null"
    );
}

#[test]
fn session_resolves_document_embeds_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><embed id='first-embed'><embed name='second-embed'></div><div id='out'></div><script>const embeds = document.embeds; const before = embeds.length; const first = embeds.namedItem('first-embed'); document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(before) + ':' + String(embeds.length) + ':' + String(first) + ':' + String(embeds.namedItem('missing'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute document.embeds scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "2:0:[object Element]:null"
    );
}

#[test]
fn session_resolves_document_embeds_iterator_helpers_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><embed id='first-embed'><embed name='second-embed'></div><div id='out'></div><script>const embeds = document.embeds; const keys = embeds.keys(); const values = embeds.values(); const entries = embeds.entries(); const firstKey = keys.next(); const firstValue = values.next(); const firstEntry = entries.next(); let serial = ''; embeds.forEach((element, index, list) => { serial += String(index) + ':' + String(element) + ':' + String(list.length) + ';'; }); document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(firstKey.value) + ':' + String(firstValue.value) + ':' + String(firstEntry.value.index) + ':' + String(firstEntry.value.value) + ':' + serial + ':' + String(embeds.length);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute document.embeds iterator helper scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "0:[object Element]:0:[object Element]:0:[object Element]:2;1:[object Element]:2;:0"
    );
}

#[test]
fn session_resolves_document_plugins_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><embed id='first-embed'><embed name='second-embed'></div><div id='out'></div><script>const plugins = document.plugins; const before = plugins.length; const first = plugins.namedItem('first-embed'); document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(before) + ':' + String(plugins.length) + ':' + String(first) + ':' + String(plugins.namedItem('missing'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute document.plugins scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "2:0:[object Element]:null"
    );
}

#[test]
fn session_resolves_document_plugins_iterator_helpers_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><embed id='first-embed'><embed name='second-embed'></div><div id='out'></div><script>const plugins = document.plugins; const keys = plugins.keys(); const values = plugins.values(); const entries = plugins.entries(); const firstKey = keys.next(); const firstValue = values.next(); const firstEntry = entries.next(); document.getElementById('out').textContent = String(firstKey.value) + ':' + firstValue.value.getAttribute('id') + ':' + String(firstEntry.value.index) + ':' + firstEntry.value.value.getAttribute('id');</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute document.plugins iterator helper scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "0:first-embed:0:first-embed"
    );
}

#[test]
fn session_resolves_document_all_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><span id='first'>First</span><span id='second'>Second</span></div><div id='out'></div><script>const all = document.all; const before = all.length; const named = all.namedItem('second'); document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(before) + ':' + String(all.length) + ':' + String(named) + ':' + String(all.namedItem('missing'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute document.all scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "5:3:[object Element]:null"
    );
}

#[test]
fn session_resolves_document_all_iterator_helpers_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><span id='first'>First</span><span id='second'>Second</span></div><div id='out'></div><script id='script'>const all = document.all; const keys = all.keys(); const values = all.values(); const entries = all.entries(); const firstKey = keys.next(); const firstValue = values.next(); const firstEntry = entries.next(); let out = ''; all.forEach((element, index, list) => { out += String(index) + ':' + element.getAttribute('id') + ':' + String(list.length) + ';'; }); document.getElementById('out').textContent = String(firstKey.value) + ':' + firstValue.value.getAttribute('id') + ':' + String(firstEntry.value.index) + ':' + firstEntry.value.value.getAttribute('id') + ':' + out;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute document.all iterator helper scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "0:root:0:root:0:root:5;1:first:5;2:second:5;3:out:5;4:script:5;"
    );
}

#[test]
fn session_resolves_get_elements_by_tag_name_ns_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><svg id='icon'><rect id='rect'></rect><circle id='dot'></circle></svg><math id='formula'><mi id='symbol'>x</mi></math><span id='label'>Label</span></div><div id='out'></div><script>const svgAll = document.getElementsByTagNameNS('http://www.w3.org/2000/svg', '*'); const svgRect = document.getElementById('icon').getElementsByTagNameNS('http://www.w3.org/2000/svg', 'rect'); const htmlSpan = document.getElementsByTagNameNS('http://www.w3.org/1999/xhtml', 'span'); const mathAll = document.getElementsByTagNameNS('http://www.w3.org/1998/Math/MathML', '*'); const beforeSvgAll = svgAll.length; const beforeSvgRect = svgRect.length; const beforeHtmlSpan = htmlSpan.length; const beforeMathAll = mathAll.length; const dot = svgAll.namedItem('dot'); document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(beforeSvgAll) + ':' + String(svgAll.length) + ':' + String(beforeSvgRect) + ':' + String(svgRect.length) + ':' + String(beforeHtmlSpan) + ':' + String(htmlSpan.length) + ':' + String(beforeMathAll) + ':' + String(mathAll.length) + ':' + String(dot) + ':' + String(svgAll.namedItem('dot'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute getElementsByTagNameNS scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "3:0:1:1:1:0:2:0:[object Element]:null"
    );
}

#[test]
fn session_reports_document_images_on_non_elements_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='wrapper'><div id='not-doc'></div></div><script>document.getElementById('not-doc').images.length;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-document images access should fail explicitly");

    assert!(error.to_string().contains("Script error"));
    assert!(error.to_string().contains("unsupported member access"));
    assert!(error.to_string().contains("`images`"));
    assert!(error.to_string().contains("element value"));
}

#[test]
fn session_reports_document_anchors_on_non_elements_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='wrapper'><div id='not-doc'></div></div><script>document.getElementById('not-doc').anchors.length;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-document anchors access should fail explicitly");

    assert!(error.to_string().contains("Script error"));
    assert!(error.to_string().contains("unsupported member access"));
    assert!(error.to_string().contains("`anchors`"));
    assert!(error.to_string().contains("element value"));
}

#[test]
fn session_resolves_window_children_through_default_view() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><span id='first'>First</span><span id='second'>Second</span></div><div id='out'></div><script>const children = document.defaultView.children; document.getElementById('out').textContent = String(children.length) + ':' + children.item(0).textContent + ':' + children.item(1).textContent + ':' + String(children.namedItem('first')) + ':' + String(children.namedItem('missing'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("window.children should resolve through defaultView");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "3:FirstSecond::null:null"
    );
}

#[test]
fn session_resolves_window_children_iterator_helpers_through_default_view() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><span id='first'>First</span><span id='second'>Second</span></div><div id='out'></div><script id='script'>const children = document.defaultView.children; const keys = children.keys(); const values = children.values(); const entries = children.entries(); const firstKey = keys.next(); const firstValue = values.next(); const firstEntry = entries.next(); let out = ''; children.forEach((element, index, list) => { out += String(index) + ':' + element.getAttribute('id') + ':' + String(list.length) + ';'; }); document.getElementById('out').textContent = String(firstKey.value) + ':' + firstValue.value.getAttribute('id') + ':' + String(firstEntry.value.index) + ':' + firstEntry.value.value.getAttribute('id') + ':' + out;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("window.children iterator helpers should resolve through defaultView");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "0:root:0:root:0:root:3;1:out:3;2:script:3;"
    );
}

#[test]
fn session_resolves_window_frames_through_default_view() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<iframe id='first' name='first'></iframe><iframe id='second'></iframe><div id='out'></div><script>const frames = document.defaultView.frames; const before = frames.length; document.getElementById('second').remove(); document.getElementById('out').textContent = String(before) + ':' + String(frames.length) + ':' + frames.item(0).getAttribute('id') + ':' + frames.namedItem('first').getAttribute('id') + ':' + String(frames.namedItem('missing'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("window.frames should resolve through defaultView");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "2:1:first:first:null"
    );
}

#[test]
fn session_resolves_window_frames_iterator_helpers_through_default_view() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<iframe id='first'></iframe><iframe id='second'></iframe><div id='out'></div><script>const frames = document.defaultView.frames; const keys = frames.keys(); const values = frames.values(); const entries = frames.entries(); const firstKey = keys.next(); const firstValue = values.next(); const firstEntry = entries.next(); let out = ''; frames.forEach((element, index, list) => { out += String(index) + ':' + element.getAttribute('id') + ':' + String(list.length) + ';'; }); document.getElementById('out').textContent = String(firstKey.value) + ':' + firstValue.value.getAttribute('id') + ':' + String(firstEntry.value.index) + ':' + firstEntry.value.value.getAttribute('id') + ':' + out;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("window.frames iterator helpers should resolve through defaultView");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "0:first:0:first:0:first:2;1:second:2;"
    );
}

#[test]
fn session_resolves_window_frame_element_through_default_view() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='out'></div><script>document.getElementById('out').textContent = String(document.defaultView.frameElement);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("window.frameElement should resolve through defaultView");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "null");
}

#[test]
fn session_resolves_window_opener_through_default_view() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='out'></div><script>document.getElementById('out').textContent = String(document.defaultView.opener);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("window.opener should resolve through defaultView");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "null");
}

#[test]
fn session_resolves_form_and_select_length_through_default_view() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><form id='signup'><input><input></form><select id='mode'><option>A</option><option>B</option></select><script>document.getElementById('out').textContent = String(document.defaultView.document.getElementById('signup').length) + ':' + String(document.defaultView.document.getElementById('mode').length);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("form.length and select.length should resolve through defaultView");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "2:2");
}

#[test]
fn session_resolves_window_length_through_default_view() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<iframe id='first' name='first'></iframe><iframe id='second'></iframe><div id='out'></div><script>const before = window.length; document.getElementById('second').remove(); document.getElementById('out').textContent = String(before) + ':' + String(window.length);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("window.length should resolve through defaultView");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "2:1");
}

#[test]
fn session_exposes_document_compat_mode() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>document.getElementById('out').textContent = document.compatMode;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("document.compatMode should resolve through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "CSS1Compat");
}

#[test]
fn session_exposes_document_character_set_and_charset_aliases() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>document.getElementById('out').textContent = document.characterSet + ':' + document.charset;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("document.characterSet should resolve through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "UTF-8:UTF-8");
}

#[test]
fn session_exposes_document_content_type() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>document.getElementById('out').textContent = document.contentType;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("document.contentType should resolve through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "text/html");
}

#[test]
fn session_exposes_document_design_mode_getter_and_setter() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>const before = document.designMode; document.designMode = 'on'; document.getElementById('out').textContent = before + ':' + document.designMode;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("document.designMode should resolve through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "off:on");
    assert_eq!(session.document_design_mode(), "on");
}

#[test]
fn session_rejects_invalid_document_design_mode_assignment() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some("<script>document.designMode = 'maybe';</script>".to_string()),
        local_storage: BTreeMap::new(),
    })
    .expect_err("document.designMode should reject unsupported values");

    assert!(error.to_string().contains("unsupported"));
    assert!(error.to_string().contains("designMode"));
}

#[test]
fn session_exposes_element_content_editable_getter_setter_and_is_content_editable() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><div id='parent' contenteditable='true'><span id='child' contenteditable='false'>Edit</span></div><script>const child = document.getElementById('child'); const before = child.contentEditable; const beforeState = child.isContentEditable; child.contentEditable = 'inherit'; const after = child.contentEditable; const afterState = child.isContentEditable; document.getElementById('out').textContent = before + ':' + String(beforeState) + ':' + after + ':' + String(afterState) + ':' + String(child.matches(':read-write'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("element.contentEditable should resolve through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "false:false:inherit:true:true"
    );
}

#[test]
fn session_exposes_element_translate_getter_setter_with_inheritance() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><div id='parent' translate='no'><span id='child'>Edit</span></div><script>const child = document.getElementById('child'); const before = String(child.translate); child.translate = true; const afterTrue = String(child.translate) + ':' + child.getAttribute('translate'); child.translate = false; const afterFalse = String(child.translate) + ':' + child.getAttribute('translate'); document.getElementById('out').textContent = before + ':' + afterTrue + ':' + afterFalse;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("element.translate should resolve through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "false:true:yes:false:no"
    );
}

#[test]
fn session_exposes_element_spellcheck_getter_setter_with_inheritance() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><div id='parent' spellcheck='false'><span id='child'>Edit</span></div><div id='mode' inputmode='text'></div><script>const child = document.getElementById('child'); const beforeSpell = String(child.spellcheck); child.spellcheck = true; const afterSpell = String(child.spellcheck) + ':' + child.getAttribute('spellcheck'); const mode = document.getElementById('mode'); const beforeMode = mode.inputMode; mode.inputMode = 'numeric'; document.getElementById('out').textContent = beforeSpell + ':' + afterSpell + ':' + beforeMode + ':' + mode.inputMode + ':' + mode.getAttribute('inputmode');</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("element.spellcheck and element.inputMode should resolve through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "false:true:true:text:numeric:numeric"
    );
}

#[test]
fn session_rejects_non_element_spellcheck_access_explicitly() {
    Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some("<main id='root'></main><script>document.createTextNode('x').spellcheck = false;</script>".to_string()),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-element spellcheck access should fail explicitly");
}

#[test]
fn session_rejects_non_element_input_mode_access_explicitly() {
    Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some("<main id='root'></main><script>document.createTextNode('x').inputMode = 'numeric';</script>".to_string()),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-element inputMode access should fail explicitly");
}

#[test]
fn session_rejects_invalid_element_content_editable_assignment() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='editable' contenteditable='true'></div><script>document.getElementById('editable').contentEditable = 'maybe';</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("element.contentEditable should reject unsupported values");

    assert!(error.to_string().contains("unsupported"));
    assert!(error.to_string().contains("contentEditable"));
}

#[test]
fn session_exposes_document_visibility_state_and_hidden() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>document.getElementById('out').textContent = document.visibilityState + ':' + String(document.hidden);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("document.visibilityState should resolve through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "visible:false");
}

#[test]
fn session_exposes_document_referrer() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>document.getElementById('out').textContent = '[' + document.referrer + ']';</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("document.referrer should resolve through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "[]");
}

#[test]
fn session_exposes_window_name_getter_and_setter() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>const before = window.name; window.self.name = 'updated'; document.getElementById('out').textContent = before + ':' + window.window.name + ':' + window.parent.name + ':' + window.top.name;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("window.name should resolve through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        ":updated:updated:updated"
    );
}

#[test]
fn session_rejects_window_self_assignment() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some("<script>window.self = 'updated';</script>".to_string()),
        local_storage: BTreeMap::new(),
    })
    .expect_err("window.self should be read-only");

    assert!(error.to_string().contains("unsupported assignment target"));
    assert!(error.to_string().contains("self"));
}

#[test]
fn session_exposes_window_closed_accessor() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>document.getElementById('out').textContent = String(window.closed) + ':' + String(window.self.closed) + ':' + String(window.window.closed) + ':' + String(window.parent.closed) + ':' + String(window.top.closed);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("window.closed should resolve through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "false:false:false:false:false"
    );
}

#[test]
fn session_rejects_window_closed_assignment() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some("<script>window.closed = true;</script>".to_string()),
        local_storage: BTreeMap::new(),
    })
    .expect_err("window.closed should be read-only");

    assert!(error.to_string().contains("unsupported assignment target"));
    assert!(error.to_string().contains("closed"));
}

#[test]
fn session_exposes_window_history_accessor() {
    let mut session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>document.getElementById('out').textContent = String(window.history) + ':' + String(window.history.length) + ':' + String(window.history.state) + ':' + String(window.history.scrollRestoration);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("window.history should resolve through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "[object History]:1:null:auto"
    );
    assert_eq!(session.window_history_length(), 1);

    session
        .navigate("https://example.test/next")
        .expect("navigation should succeed");
    assert_eq!(session.window_history_length(), 2);
}

#[test]
fn session_updates_window_history_state_via_push_and_replace_state() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>window.history.pushState('step-1', '', 'https://example.test/step-1'); window.history.replaceState('step-2', '', 'https://example.test/step-2'); document.getElementById('out').textContent = document.location + ':' + String(window.history.length) + ':' + String(window.history.state);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("window.history.pushState and replaceState should resolve through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "https://example.test/step-2:2:step-2"
    );
    assert_eq!(session.window_history_length(), 2);
    assert_eq!(session.window_history_state(), Some("step-2"));
    assert_eq!(session.document_location(), "https://example.test/step-2");
}

#[test]
fn session_rejects_window_history_assignment() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some("<script>window.history.length = 2;</script>".to_string()),
        local_storage: BTreeMap::new(),
    })
    .expect_err("window.history should be read-only");

    assert!(error.to_string().contains("history"));
    assert!(error.to_string().contains("length"));
}

#[test]
fn session_rejects_window_history_state_assignment() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some("<script>window.history.state = 'step';</script>".to_string()),
        local_storage: BTreeMap::new(),
    })
    .expect_err("window.history.state should be read-only");

    assert!(error.to_string().contains("history"));
    assert!(error.to_string().contains("state"));
}

#[test]
fn session_rejects_window_history_push_state_with_too_few_arguments() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some("<script>window.history.pushState('step');</script>".to_string()),
        local_storage: BTreeMap::new(),
    })
    .expect_err("window.history.pushState should reject too few arguments");

    assert!(
        error
            .to_string()
            .contains("history.pushState() expects 2 or 3 arguments")
    );
}

#[test]
fn session_rejects_window_history_replace_state_with_too_few_arguments() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some("<script>window.history.replaceState('step');</script>".to_string()),
        local_storage: BTreeMap::new(),
    })
    .expect_err("window.history.replaceState should reject too few arguments");

    assert!(
        error
            .to_string()
            .contains("history.replaceState() expects 2 or 3 arguments")
    );
}

#[test]
fn session_updates_window_history_scroll_restoration() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>window.history.scrollRestoration = 'manual'; document.getElementById('out').textContent = String(window.history.scrollRestoration);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("window.history.scrollRestoration should be writable");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "manual");
}

#[test]
fn session_rejects_window_history_scroll_restoration_assignment() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some("<script>window.history.scrollRestoration = 'sideways';</script>".to_string()),
        local_storage: BTreeMap::new(),
    })
    .expect_err("window.history.scrollRestoration should reject invalid values");

    assert!(error.to_string().contains("scroll restoration"));
}

#[test]
fn session_resolves_window_history_methods() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>window.location = 'https://example.test/step-1'; window.location = 'https://example.test/step-2'; window.history.back(); window.history.forward(); window.history.go(-1); document.getElementById('out').textContent = document.location + ':' + String(window.history.length);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("window.history methods should resolve through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "https://example.test/step-1:3"
    );
    assert_eq!(session.window_history_length(), 3);
    assert_eq!(session.mocks().location().navigations().len(), 5);
}

#[test]
fn session_rejects_window_history_back_arguments() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some("<script>window.history.back(1);</script>".to_string()),
        local_storage: BTreeMap::new(),
    })
    .expect_err("window.history.back should reject arguments");

    assert!(
        error
            .to_string()
            .contains("history.back() expects no arguments")
    );
}

#[test]
fn session_exposes_match_media_through_inline_scripts() {
    let mut local_storage = BTreeMap::new();
    local_storage.insert(
        "__browser_tester_match_media__(prefers-color-scheme: dark)".to_string(),
        "true".to_string(),
    );

    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>const list = window.matchMedia('(prefers-color-scheme: dark)'); document.getElementById('out').textContent = String(list.matches) + ':' + list.media + ':' + String(window.matchMedia('(prefers-color-scheme: dark)'));</script>"
                .to_string(),
        ),
        local_storage,
    })
    .expect("matchMedia should resolve through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "true:(prefers-color-scheme: dark):[object MediaQueryList]"
    );
    assert_eq!(
        session.mocks().match_media().calls(),
        &[
            bt_runtime::MatchMediaCall {
                query: "(prefers-color-scheme: dark)".to_string(),
            },
            bt_runtime::MatchMediaCall {
                query: "(prefers-color-scheme: dark)".to_string(),
            }
        ]
    );
    assert!(session.mocks().storage().local().is_empty());
}

#[test]
fn session_supports_match_media_listeners() {
    let mut local_storage = BTreeMap::new();
    local_storage.insert(
        "__browser_tester_match_media__(prefers-color-scheme: dark)".to_string(),
        "true".to_string(),
    );

    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>const out = document.getElementById('out'); out.textContent = 'before'; const list = window.matchMedia('(prefers-color-scheme: dark)'); list.addListener(() => { out.textContent = 'called'; }); list.removeListener(() => { out.textContent = 'removed'; }); out.textContent += ':' + String(list.matches) + ':' + list.media + ':' + String(window.matchMedia('(prefers-color-scheme: dark)'));</script>"
                .to_string(),
        ),
        local_storage,
    })
    .expect("matchMedia listeners should resolve through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "before:true:(prefers-color-scheme: dark):[object MediaQueryList]"
    );
    assert_eq!(session.mocks().match_media().calls().len(), 2);
    assert_eq!(
        session.mocks().match_media().listener_calls(),
        &[
            MatchMediaListenerCall {
                query: "(prefers-color-scheme: dark)".to_string(),
                method: "addListener".to_string(),
            },
            MatchMediaListenerCall {
                query: "(prefers-color-scheme: dark)".to_string(),
                method: "removeListener".to_string(),
            },
        ]
    );
}

#[test]
fn session_exposes_document_dir_getter_and_setter() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root' dir='ltr'><div id='out'></div><script>const before = document.dir; document.dir = 'rtl'; document.getElementById('out').textContent = before + ':' + document.dir + ':' + document.documentElement.getAttribute('dir');</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("document.dir should resolve through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "ltr:rtl:rtl");
}

#[test]
fn session_exposes_element_dir_and_lang_getter_and_setter() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><div id='box'></div><div id='out'></div><script>const box = document.getElementById('box'); const before = String(box.dir) + '|' + String(box.lang); box.dir = 'rtl'; box.lang = 'fr'; const afterSet = String(box.dir) + '|' + String(box.lang) + '|' + String(box.getAttribute('dir')) + '|' + String(box.getAttribute('lang')); document.getElementById('out').textContent = before + ';' + afterSet;</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("element dir and lang should resolve through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "|;rtl|fr|rtl|fr"
    );
}

#[test]
fn session_rejects_non_element_lang_access_explicitly() {
    Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='wrapper'><div id='box'></div></div><script>document.createTextNode('x').lang;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-element lang access should fail explicitly");
}

#[test]
fn session_reports_document_embeds_on_non_elements_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='wrapper'><div id='not-doc'></div></div><script>document.getElementById('not-doc').embeds.length;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-document embeds access should fail explicitly");

    assert!(error.to_string().contains("Script error"));
    assert!(error.to_string().contains("unsupported member access"));
    assert!(error.to_string().contains("`embeds`"));
    assert!(error.to_string().contains("element value"));
}

#[test]
fn session_reports_document_plugins_on_non_elements_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='wrapper'><div id='not-doc'></div></div><script>document.getElementById('not-doc').plugins.length;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-document plugins access should fail explicitly");

    assert!(error.to_string().contains("Script error"));
    assert!(error.to_string().contains("unsupported member access"));
    assert!(error.to_string().contains("`plugins`"));
    assert!(error.to_string().contains("element value"));
}

#[test]
fn session_reports_document_style_sheets_on_non_elements_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='wrapper'><div id='not-doc'></div></div><script>document.getElementById('not-doc').styleSheets.length;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-document styleSheets access should fail explicitly");

    assert!(error.to_string().contains("Script error"));
    assert!(error.to_string().contains("unsupported member access"));
    assert!(error.to_string().contains("`styleSheets`"));
    assert!(error.to_string().contains("element value"));
}

#[test]
fn session_reports_labels_on_non_labelable_elements_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='wrapper'><div id='not-labelable'></div></div><script>document.getElementById('not-labelable').labels.length;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-labelable labels access should fail explicitly");

    assert!(
        error
            .to_string()
            .contains("node is not a labelable element")
    );
}

#[test]
fn session_reports_get_elements_by_tag_name_ns_arity_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><svg id='icon'><rect id='rect'></rect></svg></div><script>document.getElementsByTagNameNS('http://www.w3.org/2000/svg');</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("arity mismatch should fail explicitly");

    assert!(error.to_string().contains("Script error"));
    assert!(
        error
            .to_string()
            .contains("getElementsByTagNameNS() expects exactly two arguments")
    );
}

#[test]
fn session_reports_form_elements_on_non_form_elements_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='wrapper'><div id='not-form'></div></div><script>document.getElementById('wrapper').elements.length;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-form elements should fail explicitly");

    assert!(error.to_string().contains("Script error"));
    assert!(error.to_string().contains("node is not a form element"));
}

#[test]
fn session_reports_select_options_on_non_select_elements_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='wrapper'><div id='not-select'></div></div><script>document.getElementById('not-select').options.length;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-select elements should fail explicitly");

    assert!(error.to_string().contains("Script error"));
    assert!(error.to_string().contains("node is not a select element"));
}

#[test]
fn session_reports_select_selected_options_on_non_select_elements_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='wrapper'><div id='not-select'></div></div><script>document.getElementById('not-select').selectedOptions.length;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-select selectedOptions access should fail explicitly");

    assert!(error.to_string().contains("Script error"));
    assert!(error.to_string().contains("node is not a select element"));
}

#[test]
fn session_supports_html_collection_for_each() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><span>child</span><span>more</span></main><div id='out'></div><script>const children = document.getElementById('root').children; children.forEach((child, index, list) => { document.getElementById('out').textContent += String(index) + ':' + child.textContent + ':' + String(list.length) + ';'; }, null);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute HTMLCollection forEach");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "0:child:2;1:more:2;"
    );
}

#[test]
fn session_supports_collection_iterator_helpers() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><span class='item'>One</span><span class='item'>Two</span></main><div id='out'></div><script>const nodes = document.querySelectorAll('.item'); const nodeValues = nodes.values(); const nodeKeys = nodes.keys(); const children = document.getElementById('root').children; const childValues = children.values(); const childKeys = children.keys(); document.getElementById('root').textContent = 'gone'; const firstNode = nodeValues.next(); const secondNode = nodeValues.next(); const thirdNode = nodeValues.next(); const firstKey = nodeKeys.next(); const secondKey = nodeKeys.next(); const thirdKey = nodeKeys.next(); const firstChild = childValues.next(); const secondChild = childValues.next(); const thirdChild = childValues.next(); const childFirstKey = childKeys.next(); const childSecondKey = childKeys.next(); const childThirdKey = childKeys.next(); document.getElementById('out').textContent = firstNode.value.textContent + ':' + String(firstNode.done) + ':' + secondNode.value.textContent + ':' + String(secondNode.done) + ':' + String(thirdNode.done) + ':' + String(firstKey.value) + ':' + String(secondKey.value) + ':' + String(thirdKey.done) + ':' + firstChild.value.textContent + ':' + String(firstChild.done) + ':' + secondChild.value.textContent + ':' + String(secondChild.done) + ':' + String(thirdChild.done) + ':' + String(childFirstKey.value) + ':' + String(childSecondKey.value) + ':' + String(childThirdKey.done);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute collection iterator helpers");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "One:false:Two:false:true:0:1:true:One:false:Two:false:true:0:1:true"
    );
}

#[test]
fn session_supports_collection_to_string_helpers() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>const childNodes = document.childNodes; document.getElementById('out').textContent = childNodes.toString() + ':' + document.plugins.toString() + ':' + window.navigator.mimeTypes.toString();</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute collection toString helpers");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "[object NodeList]:[object HTMLCollection]:[object MimeTypeArray]"
    );
}

#[test]
fn session_resolves_element_matches_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root' class='primary'><section><div id='child' class='child'></div></section></main><div id='out'></div><script>const root = document.getElementById('root'); const child = document.getElementById('child'); document.getElementById('out').textContent = String(root.matches('.primary')) + ':' + String(root.matches('.child')) + ':' + String(child.matches('.child'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute matches scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "true:false:true"
    );
}

#[test]
fn session_resolves_element_closest_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root' class='primary'>ROOT<section id='section'>SECTION<div id='child' class='child'>CHILD</div></section></main><div id='out'></div><script>const root = document.getElementById('root'); const child = document.getElementById('child'); document.getElementById('out').textContent = root.closest('.primary').textContent + ':' + child.closest('.child').textContent + ':' + child.closest('#section').textContent + ':' + String(child.closest('.missing'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute closest scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "ROOTSECTIONCHILD:CHILD:SECTIONCHILD:null"
    );
}

#[test]
fn session_writes_document_html_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<html><body><main id='root'><div id='out'></div><script>document.write('<span id=\"written\">Written</span>'); document.getElementById('out').textContent = document.getElementById('written').textContent + ':' + document.body.lastElementChild.getAttribute('id');</script></main></body></html>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute document.write scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "Written:written"
    );
    assert_eq!(session.dom().select("#written").unwrap().len(), 1);
}

#[test]
fn session_writes_document_html_with_newline_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<html><body><main id='root'><div id='out'></div><script>document.writeln('<span id=\"written\">Written</span>'); document.getElementById('out').textContent = document.getElementById('written').textContent + ':' + document.body.lastElementChild.getAttribute('id') + ':' + String(document.body.lastChild.nodeType);</script></main></body></html>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute document.writeln scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "Written:written:3"
    );
    assert_eq!(session.dom().select("#written").unwrap().len(), 1);
}

#[test]
fn session_opens_document_and_writes_fresh_html_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<html><head><title>Title</title></head><body><main id='root'><div id='out'></div></main><script>const opened = document.open(); document.write('<span id=\"written\">Written</span><div id=\"out\"></div>'); const closed = document.close(); document.getElementById('out').textContent = String(opened) + ':' + String(closed) + ':' + String(document.documentElement) + ':' + String(document.body);</script></body></html>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute document.open/document.write/document.close scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "[object Document]:[object Document]:[object Element]:null"
    );
    assert_eq!(session.dom().select("#written").unwrap().len(), 1);
    assert!(session.dom().select("#root").unwrap().is_empty());
}

#[test]
fn session_resolves_insert_adjacent_html_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><section id='target'><button id='old' class='primary'>Old</button></section></main><div id='out'></div><script>const target = document.getElementById('target'); target.insertAdjacentHTML('beforebegin', '<aside id=\"before\">Before</aside>'); target.insertAdjacentHTML('afterbegin', '<span id=\"first\">First</span>'); target.insertAdjacentHTML('beforeend', '<span id=\"last\">Last</span>'); target.insertAdjacentHTML('afterend', '<aside id=\"after\">After</aside>'); document.getElementById('out').textContent = document.getElementById('root').innerHTML + '|' + target.innerHTML + '|' + String(target.children.length) + ':' + String(document.querySelector('#before')) + ':' + String(document.querySelector('#after'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute insertAdjacentHTML scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "<aside id=\"before\">Before</aside><section id=\"target\"><span id=\"first\">First</span><button class=\"primary\" id=\"old\">Old</button><span id=\"last\">Last</span></section><aside id=\"after\">After</aside>|<span id=\"first\">First</span><button class=\"primary\" id=\"old\">Old</button><span id=\"last\">Last</span>|3:[object Element]:[object Element]"
    );
    assert_eq!(session.dom().select("#before").unwrap().len(), 1);
    assert_eq!(session.dom().select("#after").unwrap().len(), 1);
    assert_eq!(session.dom().select("#target > #first").unwrap().len(), 1);
    assert_eq!(session.dom().select("#target > #last").unwrap().len(), 1);
}

#[test]
fn session_resolves_insert_adjacent_element_and_text_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><section id='target'><button id='old' class='primary'>Old</button></section></main><div id='out'></div><script>const target = document.getElementById('target'); const before = target.insertAdjacentElement('beforebegin', document.createElement('aside')); before.setAttribute('id', 'before'); before.textContent = 'Before'; target.insertAdjacentText('afterbegin', 'First'); const last = target.insertAdjacentElement('beforeend', document.createElement('span')); last.setAttribute('id', 'last'); last.textContent = 'Last'; const after = target.insertAdjacentElement('afterend', document.createElement('aside')); after.setAttribute('id', 'after'); after.textContent = 'After'; target.insertAdjacentText('beforeend', 'Tail'); document.getElementById('out').textContent = document.getElementById('root').innerHTML + '|' + target.innerHTML + '|' + String(before) + ':' + String(after);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute insertAdjacentElement and insertAdjacentText scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "<aside id=\"before\">Before</aside><section id=\"target\">First<button class=\"primary\" id=\"old\">Old</button><span id=\"last\">Last</span>Tail</section><aside id=\"after\">After</aside>|First<button class=\"primary\" id=\"old\">Old</button><span id=\"last\">Last</span>Tail|[object Element]:[object Element]"
    );
    assert_eq!(session.dom().select("#before").unwrap().len(), 1);
    assert_eq!(session.dom().select("#after").unwrap().len(), 1);
    assert_eq!(session.dom().select("#target > #last").unwrap().len(), 1);
}

#[test]
fn session_rejects_malformed_document_write_html_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<html><body><main id='root'><div id='out'></div><script>document.write('<span></main>');</script></main></body></html>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("malformed document.write fragments should fail explicitly");

    assert!(error.to_string().contains("Script error"));
    assert!(error.to_string().contains("mismatched closing tag"));
}

#[test]
fn session_rejects_malformed_document_writeln_html_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<html><body><main id='root'><div id='out'></div><script>document.writeln('<span></main>');</script></main></body></html>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("malformed document.writeln fragments should fail explicitly");

    assert!(error.to_string().contains("Script error"));
    assert!(error.to_string().contains("mismatched closing tag"));
}

#[test]
fn session_rejects_insert_adjacent_html_on_detached_nodes_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><section id='target'><span id='old'>Old</span></section></main><script>const target = document.getElementById('target'); target.outerHTML = '<section id=\"replacement\"></section>'; target.insertAdjacentHTML('beforebegin', '<aside id=\"before\">Before</aside>');</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("detached insertAdjacentHTML should fail explicitly");

    assert!(error.to_string().contains("Script error"));
    assert!(
        error
            .to_string()
            .contains("insertAdjacentHTML(beforebegin)")
    );
}

#[test]
fn session_rejects_insert_adjacent_element_invalid_position_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><section id='target'></section><div id='out'></div><script>document.getElementById('target').insertAdjacentElement('middle', document.createElement('aside'));</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("invalid insertAdjacentElement positions should fail");

    assert!(
        error
            .to_string()
            .contains("unsupported insertAdjacentElement position")
    );
}

#[test]
fn session_rejects_insert_adjacent_text_on_void_elements_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><img id='image'></main><script>document.getElementById('image').insertAdjacentText('beforeend', 'Bad');</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("void elements should reject insertAdjacentText beforeend");

    assert!(
        error
            .to_string()
            .contains("insertAdjacentText is not supported on void elements")
    );
}

#[test]
fn session_wires_dialog_clipboard_and_location_mocks() {
    let mut session = Session::new(SessionConfig::default()).expect("session should build");

    session.mocks_mut().dialogs_mut().push_confirm(true);
    session.mocks_mut().dialogs_mut().push_prompt(Some("Ada"));
    session.mocks_mut().clipboard_mut().seed_text("seeded");

    assert_eq!(session.confirm("Continue?").unwrap(), true);
    assert_eq!(session.prompt("Name?").unwrap(), Some("Ada".to_string()));
    assert_eq!(session.read_clipboard().unwrap(), "seeded");

    session.write_clipboard("copied");
    session.alert("Notice");
    session.navigate("https://example.test/next").unwrap();

    assert_eq!(
        session.mocks().dialogs().confirm_messages(),
        &["Continue?".to_string()]
    );
    assert_eq!(
        session.mocks().dialogs().prompt_messages(),
        &["Name?".to_string()]
    );
    assert_eq!(
        session.mocks().dialogs().alert_messages(),
        &["Notice".to_string()]
    );
    assert_eq!(
        session.mocks().clipboard().writes(),
        &["copied".to_string()]
    );
    assert_eq!(session.mocks().clipboard().seeded_text(), Some("copied"));
    assert_eq!(
        session.mocks().location().current_url(),
        Some("https://example.test/next")
    );
    assert_eq!(
        session.mocks().location().navigations(),
        &["https://example.test/next".to_string()]
    );
    assert_eq!(
        session.mocks().location().current_url(),
        Some("https://example.test/next")
    );
}

#[test]
fn session_routes_window_dialogs_through_script_runtime() {
    let mut session = Session::new(SessionConfig::default()).expect("session should build");
    session.mocks_mut().dialogs_mut().push_confirm(true);
    session.mocks_mut().dialogs_mut().push_prompt(Some("Ada"));
    let mut runtime = ScriptRuntime::new();

    runtime
        .eval_program(
            "window.alert('Notice'); window.confirm('Continue?'); window.prompt('Name?', 'Default');",
            "inline-script",
            &mut session,
        )
        .expect("window dialogs should resolve through the session host bindings");

    assert_eq!(
        session.mocks().dialogs().alert_messages(),
        &["Notice".to_string()]
    );
    assert_eq!(
        session.mocks().dialogs().confirm_messages(),
        &["Continue?".to_string()]
    );
    assert_eq!(
        session.mocks().dialogs().prompt_messages(),
        &["Name?".to_string()]
    );
    assert!(session.mocks().dialogs().confirm_queue().is_empty());
    assert!(session.mocks().dialogs().prompt_queue().is_empty());
}

#[test]
fn session_print_records_calls_through_the_registry() {
    let mut session = Session::new(SessionConfig::default()).expect("session should build");

    session.print().expect("print should succeed by default");

    assert_eq!(session.mocks().print().calls().len(), 1);
}

#[test]
fn session_exposes_window_navigator_metadata() {
    let session = Session::new(SessionConfig::default()).expect("session should build");

    assert_eq!(session.window_navigator_user_agent(), "browser-tester-next");
    assert_eq!(
        session.window_navigator_app_code_name(),
        "browser-tester-next"
    );
    assert_eq!(session.window_navigator_app_name(), "browser-tester-next");
    assert_eq!(
        session.window_navigator_app_version(),
        "browser-tester-next"
    );
    assert_eq!(session.window_navigator_product(), "browser-tester-next");
    assert_eq!(
        session.window_navigator_product_sub(),
        "browser-tester-next"
    );
    assert_eq!(session.window_navigator_vendor(), "browser-tester-next");
    assert_eq!(session.window_navigator_vendor_sub(), "browser-tester-next");
    assert!(!session.window_navigator_pdf_viewer_enabled());
    assert_eq!(session.window_navigator_do_not_track(), "unspecified");
    assert!(!session.window_navigator_java_enabled());
    assert_eq!(session.window_navigator_platform(), "unknown");
    assert_eq!(session.window_navigator_language(), "en-US");
    assert_eq!(session.window_navigator_oscpu(), "unknown");
    assert_eq!(session.window_navigator_system_language(), "en-US");
    assert_eq!(
        session.window_navigator_languages(),
        vec!["en-US".to_string()]
    );
    assert!(session.window_navigator_mime_types().is_empty());
    assert!(session.window_navigator_cookie_enabled());
    assert!(session.window_navigator_on_line());
    assert!(!session.window_navigator_webdriver());
    assert_eq!(session.window_navigator_hardware_concurrency(), 8);
    assert_eq!(session.window_navigator_max_touch_points(), 0);
}

#[test]
fn session_exposes_window_navigator_clipboard_access() {
    let mut session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some("<div id='out'></div>".to_string()),
        local_storage: BTreeMap::new(),
    })
    .expect("session should build");
    let mut runtime = ScriptRuntime::new();

    runtime
        .eval_program(
            "window.navigator.clipboard.writeText('copied'); document.getElementById('out').textContent = String(window.navigator.clipboard) + ':' + window.navigator.clipboard.readText();",
            "inline-script",
            &mut session,
        )
        .expect("window.navigator.clipboard should resolve through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "[object Clipboard]:copied"
    );
    assert_eq!(
        session.mocks().clipboard().writes(),
        &["copied".to_string()]
    );
    assert_eq!(session.mocks().clipboard().seeded_text(), Some("copied"));
}

#[test]
fn session_exposes_window_navigator_languages_alias() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='out'></div><script>const languages = window.navigator.languages; document.getElementById('out').textContent = window.navigator.userLanguage + ':' + window.navigator.browserLanguage + ':' + window.navigator.systemLanguage + ':' + window.navigator.oscpu + ':' + String(languages.length) + ':' + languages.item(0) + ':' + languages.toString() + ':' + String(languages.contains('en-US')) + ':' + String(languages.contains('fr-FR'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose window.navigator.languages");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "en-US:en-US:en-US:unknown:1:en-US:[object DOMStringList]:true:false"
    );
}

#[test]
fn session_exposes_window_navigator_languages_iterator_helpers() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='out'></div><script>const languages = window.navigator.languages; const keys = languages.keys(); const values = languages.values(); const entries = languages.entries(); const firstKey = keys.next(); const firstValue = values.next(); const firstEntry = entries.next(); const secondKey = keys.next(); const secondValue = values.next(); const secondEntry = entries.next(); document.getElementById('out').textContent = String(firstKey.value) + ':' + String(firstValue.value) + ':' + String(firstEntry.value.index) + ':' + firstEntry.value.value + ':' + String(secondKey.done) + ':' + String(secondValue.done) + ':' + String(secondEntry.done);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose window.navigator.languages iterator helpers");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "0:en-US:0:en-US:true:true:true"
    );
}

#[test]
fn session_exposes_window_navigator_plugins_alias() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><embed id='first-embed'><embed name='second-embed'></div><div id='out'></div><script>document.getElementById('out').textContent = String(window.navigator.plugins.length) + ':' + String(window.navigator.plugins.namedItem('first-embed')) + ':' + String(window.navigator.plugins.namedItem('missing'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose window.navigator.plugins");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "2:[object Element]:null"
    );
}

#[test]
fn session_exposes_window_navigator_plugins_iterator_helpers() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><embed id='first-embed'><embed id='second-embed'></div><div id='out'></div><script>const plugins = window.navigator.plugins; const keys = plugins.keys(); const values = plugins.values(); const entries = plugins.entries(); const firstKey = keys.next(); const firstValue = values.next(); const firstEntry = entries.next(); let out = ''; plugins.forEach((element, index, list) => { out += String(index) + ':' + element.getAttribute('id') + ':' + String(list.length) + ';'; }); document.getElementById('out').textContent = String(firstKey.value) + ':' + firstValue.value.getAttribute('id') + ':' + String(firstEntry.value.index) + ':' + firstEntry.value.value.getAttribute('id') + ':' + out;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose window.navigator.plugins iterator helpers");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "0:first-embed:0:first-embed:0:first-embed:2;1:second-embed:2;"
    );
}

#[test]
fn session_exposes_window_navigator_plugins_refresh() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='out'></div><script>document.getElementById('out').textContent = String(document.plugins.refresh()) + ':' + String(window.navigator.plugins.refresh());</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose window.navigator.plugins.refresh");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "undefined:undefined"
    );
}

#[test]
fn session_exposes_window_navigator_mime_types_alias() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='out'></div><script>document.getElementById('out').textContent = String(window.navigator.mimeTypes.length) + ':' + String(window.navigator.mimeTypes.item(0)) + ':' + String(window.navigator.mimeTypes.namedItem('missing'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose window.navigator.mimeTypes");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "0:null:null");
}

#[test]
fn session_exposes_window_navigator_mime_types_iterator_helpers() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='out'></div><script>const mimeTypes = window.navigator.mimeTypes; const keys = mimeTypes.keys(); const values = mimeTypes.values(); const entries = mimeTypes.entries(); const firstKey = keys.next(); const firstValue = values.next(); const firstEntry = entries.next(); document.getElementById('out').textContent = String(firstKey.done) + ':' + String(firstValue.done) + ':' + String(firstEntry.done);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose window.navigator.mimeTypes iterator helpers");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "true:true:true"
    );
}

#[test]
fn session_exposes_window_navigator_collection_helpers_for_each() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='out'></div><script>const languages = window.navigator.languages; const mimeTypes = window.navigator.mimeTypes; languages.forEach((value, index, list) => { document.getElementById('out').textContent += String(index) + ':' + value + ':' + String(list.length) + ';'; }); mimeTypes.forEach(() => { document.getElementById('out').textContent = 'called'; });</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose window.navigator collection forEach helpers");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "0:en-US:1;");
}

#[test]
fn session_exposes_named_node_map_for_each() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><div id='box' data-role='menu' data-state='azure'></div><div id='out'></div><script>const attrs = document.getElementById('box').attributes; let out = ''; attrs.forEach((attr, index, list) => { out += String(index) + ':' + attr.name + ':' + attr.value + ':' + String(list.length) + ';'; }); document.getElementById('out').textContent = out;</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose NamedNodeMap.forEach");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "0:data-role:menu:3;1:data-state:azure:3;2:id:box:3;"
    );
}

#[test]
fn session_exposes_named_node_map_iterators() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><div id='box' data-role='menu' data-state='azure'></div><div id='out'></div><script>const attrs = document.getElementById('box').attributes; const keys = attrs.keys(); const values = attrs.values(); const entries = attrs.entries(); const firstKey = keys.next(); const firstValue = values.next(); const firstEntry = entries.next(); document.getElementById('out').textContent = String(attrs) + ':' + String(firstKey.value) + ':' + firstValue.value.name + ':' + String(firstEntry.value.index) + ':' + firstEntry.value.value.name;</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose NamedNodeMap iterator helpers");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "[object NamedNodeMap]:0:data-role:0:data-role"
    );
}

#[test]
fn session_exposes_named_node_map_iterator_helpers_exhaustion() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><div id='box' data-role='menu' data-state='azure'></div><div id='out'></div><script>const attrs = document.getElementById('box').attributes; const keys = attrs.keys(); const values = attrs.values(); const entries = attrs.entries(); const firstKey = keys.next(); const secondKey = keys.next(); const thirdKey = keys.next(); const firstValue = values.next(); const secondValue = values.next(); const thirdValue = values.next(); const firstEntry = entries.next(); const secondEntry = entries.next(); const thirdEntry = entries.next(); document.getElementById('out').textContent = String(attrs) + ':' + String(firstKey.value) + ':' + String(secondKey.value) + ':' + String(thirdKey.done) + ':' + firstValue.value.name + ':' + secondValue.value.name + ':' + String(thirdValue.done) + ':' + String(firstEntry.value.index) + ':' + firstEntry.value.value.name + ':' + String(secondEntry.value.index) + ':' + secondEntry.value.value.name + ':' + String(thirdEntry.done);</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose NamedNodeMap iterator helper exhaustion");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "[object NamedNodeMap]:0:1:false:data-role:data-state:false:0:data-role:1:data-state:false"
    );
}

#[test]
fn session_exposes_window_device_pixel_ratio() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>document.getElementById('out').textContent = String(window.devicePixelRatio);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose window.devicePixelRatio");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "1");
    assert_eq!(session.window_device_pixel_ratio(), 1.0);
}

#[test]
fn session_exposes_window_inner_width_and_inner_height() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>document.getElementById('out').textContent = String(window.innerWidth) + ':' + String(window.innerHeight);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose window.innerWidth and window.innerHeight");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "1024:768");
    assert_eq!(session.window_inner_width(), 1024);
    assert_eq!(session.window_inner_height(), 768);
}

#[test]
fn session_exposes_window_outer_width_and_outer_height() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>document.getElementById('out').textContent = String(window.outerWidth) + ':' + String(window.outerHeight);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose window.outerWidth and window.outerHeight");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "1024:768");
    assert_eq!(session.window_outer_width(), 1024);
    assert_eq!(session.window_outer_height(), 768);
}

#[test]
fn session_exposes_window_screen_position() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>document.getElementById('out').textContent = String(window.screenX) + ':' + String(window.screenY) + ':' + String(window.screenLeft) + ':' + String(window.screenTop);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose window.screenX / screenY / screenLeft / screenTop");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "0:0:0:0");
    assert_eq!(session.window_screen_x(), 0);
    assert_eq!(session.window_screen_y(), 0);
    assert_eq!(session.window_screen_left(), 0);
    assert_eq!(session.window_screen_top(), 0);
}

#[test]
fn session_exposes_window_screen_object_metadata() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>document.getElementById('out').textContent = String(window.screen) + ':' + String(window.screen.width) + ':' + String(window.screen.height) + ':' + String(window.screen.availWidth) + ':' + String(window.screen.availHeight) + ':' + String(window.screen.availLeft) + ':' + String(window.screen.availTop) + ':' + String(window.screen.colorDepth) + ':' + String(window.screen.pixelDepth);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose window.screen metadata");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "[object Screen]:1024:768:1024:768:0:0:24:24"
    );
    assert_eq!(session.window_screen_width(), 1024);
    assert_eq!(session.window_screen_height(), 768);
    assert_eq!(session.window_screen_avail_width(), 1024);
    assert_eq!(session.window_screen_avail_height(), 768);
    assert_eq!(session.window_screen_avail_left(), 0);
    assert_eq!(session.window_screen_avail_top(), 0);
    assert_eq!(session.window_screen_color_depth(), 24);
    assert_eq!(session.window_screen_pixel_depth(), 24);
}

#[test]
fn session_exposes_window_screen_orientation_metadata() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>document.getElementById('out').textContent = String(window.screen.orientation) + ':' + window.screen.orientation.type + ':' + String(window.screen.orientation.angle);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose window.screen.orientation metadata");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "[object ScreenOrientation]:landscape-primary:0"
    );
}

#[test]
fn session_scroll_records_calls_through_the_registry() {
    let mut session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>window.scrollTo(10, 20); document.getElementById('out').textContent = 'done';</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should build");

    session
        .scroll_by(-5, 3)
        .expect("scroll should succeed by default");

    assert_eq!(session.mocks().scroll().calls().len(), 2);
    assert_eq!(
        session.mocks().scroll().calls()[0],
        bt_runtime::ScrollCall {
            method: ScrollMethod::To,
            x: 10,
            y: 20,
        }
    );
    assert_eq!(
        session.mocks().scroll().calls()[1],
        bt_runtime::ScrollCall {
            method: ScrollMethod::By,
            x: -5,
            y: 3,
        }
    );
    assert_eq!(session.window_scroll_x(), 5);
    assert_eq!(session.window_scroll_y(), 23);
    assert_eq!(session.window_page_x_offset(), 5);
    assert_eq!(session.window_page_y_offset(), 23);
}

#[test]
fn session_close_records_calls_through_the_registry() {
    let mut session = Session::new(SessionConfig::default()).expect("session should build");

    session.close().expect("close should succeed by default");

    assert_eq!(session.mocks().close().calls().len(), 1);
}

#[test]
fn session_open_records_calls_through_the_registry() {
    let mut session = Session::new(SessionConfig::default()).expect("session should build");

    session
        .open(
            Some("https://example.test/popup"),
            Some("_blank"),
            Some("noopener"),
        )
        .expect("open should succeed by default");

    assert_eq!(session.mocks().open().calls().len(), 1);
    assert_eq!(
        session.mocks().open().calls()[0].url.as_deref(),
        Some("https://example.test/popup")
    );
    assert_eq!(
        session.mocks().open().calls()[0].target.as_deref(),
        Some("_blank")
    );
    assert_eq!(
        session.mocks().open().calls()[0].features.as_deref(),
        Some("noopener")
    );
}

#[test]
fn session_rejects_close_failure_seed_during_bootstrap() {
    let mut local_storage = BTreeMap::new();
    local_storage.insert(
        "__browser_tester_close_failure__".to_string(),
        "window closed".to_string(),
    );

    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some("<main id='out'></main><script>window.close();</script>".to_string()),
        local_storage,
    })
    .expect_err("close failure seed should fail bootstrap when window.close runs");

    assert!(error.to_string().contains("window closed"));
}

#[test]
fn session_rejects_open_failure_seed_during_bootstrap() {
    let mut local_storage = BTreeMap::new();
    local_storage.insert(
        "__browser_tester_open_failure__".to_string(),
        "popup blocked".to_string(),
    );

    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>window.open('https://example.test/popup');</script>"
                .to_string(),
        ),
        local_storage,
    })
    .expect_err("open failure seed should fail bootstrap when window.open runs");

    assert!(error.to_string().contains("popup blocked"));
}

#[test]
fn session_rejects_scroll_failure_seed_during_bootstrap() {
    let mut local_storage = BTreeMap::new();
    local_storage.insert(
        "__browser_tester_scroll_failure__".to_string(),
        "scroll blocked".to_string(),
    );

    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some("<main id='out'></main><script>window.scrollTo(10, 20);</script>".to_string()),
        local_storage,
    })
    .expect_err("scroll failure seed should fail bootstrap when window.scrollTo runs");

    assert!(error.to_string().contains("scroll blocked"));
}

#[test]
fn session_fetch_uses_mock_registry_and_reports_missing_rules() {
    let mut session = Session::new(SessionConfig::default()).expect("session should build");

    session
        .mocks_mut()
        .fetch_mut()
        .respond_text("https://example.test/api/message", 201, "ok");
    session
        .mocks_mut()
        .fetch_mut()
        .fail("https://example.test/api/error", "network disabled");

    let response = session
        .fetch("https://example.test/api/message")
        .expect("fetch should use mock response");
    assert_eq!(response.url, "https://example.test/api/message");
    assert_eq!(response.status, 201);
    assert_eq!(response.body, "ok");
    assert_eq!(session.mocks().fetch().calls().len(), 1);
    assert_eq!(
        session.mocks().fetch().calls()[0].url,
        "https://example.test/api/message"
    );

    let error = session
        .fetch("https://example.test/api/error")
        .expect_err("mocked fetch failure should propagate");
    assert!(error.to_string().contains("network disabled"));

    let missing = session
        .fetch("https://example.test/api/missing")
        .expect_err("missing fetch mock should fail");
    assert!(
        missing
            .to_string()
            .contains("no fetch mock configured for `https://example.test/api/missing`")
    );
}

#[test]
fn session_rejects_print_failure_seed_during_bootstrap() {
    let mut local_storage = BTreeMap::new();
    local_storage.insert(
        "__browser_tester_print_failure__".to_string(),
        "print blocked".to_string(),
    );

    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some("<main id='out'></main><script>window.print();</script>".to_string()),
        local_storage,
    })
    .expect_err("print failure seed should fail bootstrap when print is called");

    assert!(error.to_string().contains("print blocked"));
}

#[test]
fn session_capture_download_records_artifacts() {
    let mut session = Session::new(SessionConfig::default()).expect("session should build");

    session
        .capture_download("report.csv", b"downloaded bytes".to_vec())
        .expect("download capture should succeed");

    assert_eq!(session.mocks().downloads().artifacts().len(), 1);
    assert_eq!(
        session.mocks().downloads().artifacts()[0].file_name,
        "report.csv"
    );
    assert_eq!(
        session.mocks().downloads().artifacts()[0].bytes,
        b"downloaded bytes".to_vec()
    );
}

#[test]
fn session_rejects_blank_download_names() {
    let mut session = Session::new(SessionConfig::default()).expect("session should build");

    let error = session
        .capture_download(" ", b"downloaded bytes".to_vec())
        .expect_err("blank download names should fail");
    assert!(
        error
            .to_string()
            .contains("capture_download() requires a non-empty file name")
    );
}

#[test]
fn session_rejects_unseeded_mock_dialogs_and_clipboard_reads() {
    let mut session = Session::new(SessionConfig::default()).expect("session should build");

    let confirm_error = session
        .confirm("Continue?")
        .expect_err("confirm should require a queued response");
    assert!(
        confirm_error
            .to_string()
            .contains("confirm() requires a queued response")
    );

    let prompt_error = session
        .prompt("Name?")
        .expect_err("prompt should require a queued response");
    assert!(
        prompt_error
            .to_string()
            .contains("prompt() requires a queued response")
    );

    let clipboard_error = session
        .read_clipboard()
        .expect_err("clipboard reads should require a seed");
    assert!(
        clipboard_error
            .to_string()
            .contains("clipboard text has not been seeded")
    );
}

#[test]
fn session_sets_file_input_files_and_dispatches_change_events() {
    let mut session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<input id='upload' type='file'><div id='out'></div><script>document.getElementById('upload').addEventListener('change', () => { document.getElementById('out').textContent = document.getElementById('upload').value; });</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should build");

    let upload_id = session.dom().select("#upload").unwrap()[0];
    let out_id = session.dom().select("#out").unwrap()[0];

    session
        .set_files_node(upload_id, "#upload", ["report.csv"])
        .expect("file selection should be accepted");

    assert_eq!(session.dom().value_for_node(upload_id), "report.csv");
    assert_eq!(session.dom().text_content_for_node(out_id), "report.csv");
    assert_eq!(
        session.mocks().file_input().selections()[0].selector,
        "#upload"
    );
    assert_eq!(
        session.mocks().file_input().selections()[0].files,
        vec!["report.csv".to_string()]
    );
}

#[test]
fn session_registers_function_listener_with_this_binding() {
    let mut session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<button id='button' data-value='ok'>go</button><div id='out'></div><script>const button = document.getElementById('button'); const out = document.getElementById('out'); button.addEventListener('click', function () { out.textContent = this.getAttribute('data-value'); });</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should build");

    let button_id = session.dom().select("#button").unwrap()[0];
    let out_id = session.dom().select("#out").unwrap()[0];

    session
        .click_node(button_id)
        .expect("click should dispatch function listener");

    assert_eq!(session.dom().text_content_for_node(out_id), "ok");
}

#[test]
fn session_rejects_set_files_on_non_file_input() {
    let mut session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some("<input id='name'>".to_string()),
        local_storage: BTreeMap::new(),
    })
    .expect("session should build");

    let name_id = session.dom().select("#name").unwrap()[0];
    let error = session
        .set_files_node(name_id, "#name", ["report.csv"])
        .expect_err("set_files should reject non-file inputs");
    assert!(error.to_string().contains("file input control"));
}

#[test]
fn session_supports_attribute_nodes_and_named_node_map() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><div id='box' data-role='menu'></div><div id='out'></div><script>const box = document.getElementById('box'); const attrs = box.attributes; const named = attrs.getNamedItem('data-role'); const created = document.createAttribute('data-state'); created.value = 'open'; const before = attrs.length; const previous = attrs.setNamedItem(created); const during = attrs.length; const snapshot = box.getAttributeNode('data-state'); const snapshotOwner = snapshot.ownerElement; const createdOwner = created.ownerElement; const removed = attrs.removeNamedItem('data-state'); const removedOwner = removed.ownerElement; const after = attrs.length; document.getElementById('out').textContent = String(before) + ':' + String(previous) + ':' + String(during) + ':' + String(snapshot) + ':' + snapshot.name + ':' + snapshot.value + ':' + String(snapshotOwner) + ':' + String(createdOwner) + ':' + String(removed) + ':' + String(removedOwner) + ':' + String(after) + ':' + String(box.getAttributeNode('data-state')) + ':' + named.value + ':' + String(named.ownerElement);</script></main>".to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should build");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "2:null:3:[object Attr]:data-state:open:[object Element]:[object Element]:[object Attr]:null:2:null:menu:[object Element]"
    );
}

#[test]
fn session_exposes_attribute_specified() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><div id='box' data-role='menu'></div><div id='out'></div><script>const detached = document.createAttribute('data-state'); const attached = document.getElementById('box').getAttributeNode('data-role'); document.getElementById('out').textContent = String(detached.specified) + ':' + String(attached.specified);</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should build");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "true:true");
}

#[test]
fn session_exposes_attribute_is_id() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><div id='box'></div><div id='out'></div><script>const detached = document.createAttribute('data-state'); const attached = document.getElementById('box').getAttributeNode('id'); document.getElementById('out').textContent = String(detached.isId) + ':' + String(attached.isId);</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should build");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "false:true");
}

#[test]
fn session_supports_optional_chaining_member_calls_explicitly() {
    let mut session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<button id='btn'>run</button><p id='out'></p><script>const actionEls = { close: document.getElementById('btn') }; actionEls.close?.addEventListener('click', () => { document.getElementById('out').textContent = 'ok'; });</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should build");

    let button_id = session.dom().select("#btn").unwrap()[0];
    let out_id = session.dom().select("#out").unwrap()[0];

    session
        .click_node(button_id)
        .expect("click should trigger the optional-chaining listener");

    assert_eq!(session.dom().text_content_for_node(out_id), "ok");
}

#[test]
fn session_supports_dom_constructors_and_instanceof_checks_explicitly() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<button id='btn'>run</button><select id='sel'><option>One</option></select><p id='out'></p><script>const button = document.getElementById('btn'); const select = document.getElementById('sel'); document.getElementById('out').textContent = [typeof HTMLButtonElement, String(window.HTMLButtonElement === HTMLButtonElement), String(button instanceof HTMLButtonElement), String(button instanceof HTMLElement), String(button instanceof Element), String(document instanceof Node), String(select instanceof HTMLSelectElement), String(window.HTMLSelectElement === HTMLSelectElement)].join('|');</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should build");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "function|true|true|true|true|true|true|true"
    );
}

#[test]
fn session_supports_number_to_fixed_explicitly() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><div id='out'></div><script>document.getElementById('out').textContent = Number(12.345).toFixed(2);</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should build");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "12.35");
}

#[test]
fn session_supports_number_to_precision_and_exponential_explicitly() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><div id='out'></div><script>const parts = [Number(12.345).toPrecision(4), Math.max(0, 4.2).toExponential(2), Number(10000).toPrecision(10)]; document.getElementById('out').textContent = parts.join('|');</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should build");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "12.35|4.20e+0|10000.00000"
    );
}

#[test]
fn session_supports_regex_literals_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><div id='out'></div><script>const template = '{name}-{count}'; const values = { name: 'Ada', count: '3' }; const replaced = template.replace(/\\{(\\w+)\\}/g, (_, key) => { return values[key]; }); const lookahead = '1000000'.replace(/\\B(?=(\\d{3})+(?!\\d))/g, ','); document.getElementById('out').textContent = replaced + '|' + lookahead;</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should build");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "Ada-3|1,000,000"
    );
}
