use std::collections::BTreeMap;

use bt_runtime::{ScrollMethod, Session, SessionConfig};

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
            "<html id='html'><head id='head'><title>Title</title></head><body id='body'><main id='out'></main><script>const html = document.documentElement; const head = document.head; const body = document.body; document.getElementById('out').textContent = html.getAttribute('id') + ':' + head.getAttribute('id') + ':' + body.getAttribute('id');</script></body></html>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose document root/head/body");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "html:head:body"
    );
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
            "<main id='root'><span id='child'></span></main><div id='out'></div><script>const root = document.getElementById('root'); const child = document.getElementById('child'); document.getElementById('out').textContent = document.origin + ':' + window.origin + ':' + root.origin + ':' + child.origin;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should expose origin aliases");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "https://example.test:8443:https://example.test:8443:https://example.test:8443:https://example.test:8443"
    );
    assert_eq!(session.document_origin(), "https://example.test:8443");
}

#[test]
fn session_resolves_web_storage_through_inline_scripts() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>const local = window.localStorage; const session = document.defaultView.sessionStorage; const before = String(local) + ':' + String(session) + ':' + String(local.length) + ':' + String(session.length); const token = local.getItem('token'); local.setItem('theme', 'dark'); local.removeItem('token'); session.setItem('scratch', 'xyz'); const sessionKey = session.key(0); session.clear(); document.getElementById('out').textContent = before + '|' + token + ':' + local.getItem('theme') + ':' + String(local.length) + ':' + String(local.key(0)) + ':' + String(session.length) + ':' + String(sessionKey);</script>"
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
        "[object Storage]:[object Storage]:1:0|abc:dark:1:theme:0:scratch"
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
            "<main id='root'><button id='button' class='base' data-kind='App'>First</button><div id='out'></div><script>const button = document.getElementById('button'); button.className = 'primary secondary'; const before = button.classList.length; const contains = button.classList.contains('primary'); button.classList.add('tertiary'); button.classList.remove('secondary'); const toggled = button.classList.toggle('active'); button.dataset.userId = '42'; document.getElementById('out').textContent = button.className + ':' + String(before) + ':' + String(contains) + ':' + String(toggled) + ':' + button.dataset.kind + ':' + button.dataset.userId + ':' + String(button.classList) + ':' + String(button.dataset);</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute class view scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "primary tertiary active:2:true:true:App:42:[object DOMTokenList]:[object DOMStringMap]"
    );
    assert_eq!(session.dom().select(".active").unwrap().len(), 1);
    assert_eq!(session.dom().select("[data-user-id]").unwrap().len(), 1);
    assert_eq!(session.dom().select("[data-kind=App]").unwrap().len(), 1);
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
            "<div id='root'><form id='signup' name='signup'>Signup</form><form id='login' name='login'>Login</form></div><div id='out'></div><script>const forms = document.forms; const first = forms.item(0); const named = forms.namedItem('signup'); const before = forms.length; const firstText = first.textContent; const namedText = named.textContent; document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(before) + ':' + String(forms.length) + ':' + firstText + ':' + namedText + ':' + String(forms.namedItem('missing'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should execute document.forms scripts");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "2:0:Signup:Signup:null"
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
            "<main id='out'></main><script>const before = window.name; window.name = 'updated'; document.getElementById('out').textContent = before + ':' + document.defaultView.name;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("window.name should resolve through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), ":updated");
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
fn session_print_records_calls_through_the_registry() {
    let mut session = Session::new(SessionConfig::default()).expect("session should build");

    session.print().expect("print should succeed by default");

    assert_eq!(session.mocks().print().calls().len(), 1);
}

#[test]
fn session_exposes_window_navigator_metadata() {
    let session = Session::new(SessionConfig::default()).expect("session should build");

    assert_eq!(session.window_navigator_user_agent(), "browser-tester-next");
    assert_eq!(session.window_navigator_app_code_name(), "browser-tester-next");
    assert_eq!(session.window_navigator_app_name(), "browser-tester-next");
    assert_eq!(
        session.window_navigator_app_version(),
        "browser-tester-next"
    );
    assert_eq!(session.window_navigator_product(), "browser-tester-next");
    assert_eq!(session.window_navigator_vendor(), "browser-tester-next");
    assert_eq!(session.window_navigator_platform(), "unknown");
    assert_eq!(session.window_navigator_language(), "en-US");
    assert!(session.window_navigator_cookie_enabled());
    assert!(session.window_navigator_on_line());
    assert!(!session.window_navigator_webdriver());
    assert_eq!(session.window_navigator_hardware_concurrency(), 8);
    assert_eq!(session.window_navigator_max_touch_points(), 0);
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
