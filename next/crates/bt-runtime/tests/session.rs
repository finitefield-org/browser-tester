use std::collections::BTreeMap;

use bt_runtime::{Session, SessionConfig};

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
fn session_reports_html_collection_method_failures_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><span>child</span></main><script>document.getElementById('root').children.forEach(() => {});</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("session should reject unsupported HTMLCollection methods");

    assert!(error.to_string().contains("Script error"));
    assert!(
        error
            .to_string()
            .contains("unsupported HTMLCollection method: forEach")
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
