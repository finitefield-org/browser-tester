use browser_tester::Harness;

#[test]
fn click_events_bubble_beyond_the_target_phase() -> browser_tester::Result<()> {
    let mut harness = Harness::from_html(
        "<div id='parent'><div id='child'></div></div><div id='out'></div><script>document.getElementById('child').addEventListener('click', () => { document.getElementById('out').textContent = 'target'; }); document.getElementById('parent').addEventListener('click', () => { document.getElementById('out').textContent += ':parent'; }); document.addEventListener('click', () => { document.getElementById('out').textContent += ':document'; }); window.addEventListener('click', () => { document.getElementById('out').textContent += ':window'; });</script>",
    )?;

    harness.click("#child")?;
    harness.assert_text("#out", "target:parent:document:window")?;
    Ok(())
}

#[test]
fn prevent_default_cancels_click_default_action() -> browser_tester::Result<()> {
    let mut harness = Harness::from_html(
        "<input id='agree' type='checkbox'><div id='out'></div><script>document.getElementById('agree').addEventListener('click', (event) => { event.preventDefault(); }); document.getElementById('agree').addEventListener('change', () => { document.getElementById('out').textContent = String(document.getElementById('agree').checked); });</script>",
    )?;

    harness.click("#agree")?;
    harness.assert_checked("#agree", false)?;
    harness.assert_text("#out", "")?;
    Ok(())
}

#[test]
fn focus_and_blur_are_publicly_supported() -> browser_tester::Result<()> {
    let mut harness = Harness::from_html(
        "<input id='first'><input id='second'><div id='out'></div><script>document.getElementById('first').addEventListener('blur', () => { document.getElementById('second').textContent = 'after-blur'; }); document.getElementById('second').addEventListener('focus', () => { document.getElementById('out').textContent = document.getElementById('second').textContent; });</script>",
    )?;

    harness.focus("#first")?;
    harness.focus("#second")?;
    harness.assert_text("#out", "after-blur")?;
    Ok(())
}

#[test]
fn element_click_focus_and_blur_are_publicly_supported() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<input id='agree' type='checkbox'><input id='first'><div id='out'></div><script>const checkbox = document.getElementById('agree'); const first = document.getElementById('first'); checkbox.click(); first.focus(); const focused = document.activeElement.getAttribute('id'); first.blur(); document.getElementById('out').textContent = String(checkbox.checked) + ':' + String(focused);</script>",
    )?;

    harness.assert_text("#out", "true:first")?;
    Ok(())
}

#[test]
fn navigator_app_name_is_publicly_supported() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<div id='out'></div><script>document.getElementById('out').textContent = window.navigator.userAgent + ':' + window.navigator.appCodeName + ':' + window.navigator.appName + ':' + window.navigator.appVersion + ':' + window.navigator.product + ':' + window.navigator.productSub + ':' + window.navigator.vendor + ':' + window.navigator.vendorSub + ':' + String(window.navigator.pdfViewerEnabled) + ':' + window.navigator.doNotTrack + ':' + String(window.navigator.javaEnabled()) + ':' + window.navigator.platform + ':' + window.navigator.language + ':' + String(window.navigator.cookieEnabled) + ':' + String(window.navigator.onLine) + ':' + String(window.navigator.webdriver);</script>",
    )?;

    harness.assert_text(
        "#out",
        "browser_tester:browser_tester:browser_tester:browser_tester:browser_tester:browser_tester:browser_tester:browser_tester:false:unspecified:false:unknown:en-US:true:true:false",
    )?;
    Ok(())
}

#[test]
fn document_cookie_is_publicly_supported() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<div id='out'></div><script>document.cookie = 'theme=dark'; document.cookie = 'theme=light'; document.getElementById('out').textContent = document.cookie;</script>",
    )?;

    harness.assert_text("#out", "theme=light")?;
    Ok(())
}

#[test]
fn document_cookie_assignment_rejects_malformed_input() -> browser_tester::Result<()> {
    let error = Harness::from_html("<script>document.cookie = 'badcookie';</script>")
        .expect_err("malformed cookie assignments should fail");

    assert!(
        error
            .to_string()
            .contains("document.cookie requires `name=value`")
    );
    Ok(())
}

#[test]
fn set_select_value_updates_selection_and_fires_change() -> browser_tester::Result<()> {
    let mut harness = Harness::from_html(
        "<select id='mode'><option value='a'>A</option><option value='b'>B</option></select><div id='out'></div><script>document.getElementById('mode').addEventListener('change', () => { document.getElementById('out').textContent = document.getElementById('mode').value; });</script>",
    )?;

    harness.set_select_value("#mode", "b")?;
    harness.assert_value("#mode", "b")?;
    harness.assert_text("#out", "b")?;
    Ok(())
}

#[test]
fn fetch_uses_mock_response_and_records_calls() -> browser_tester::Result<()> {
    let mut harness = Harness::builder().build()?;

    harness
        .mocks_mut()
        .fetch()
        .respond_text("https://example.test/api/message", 201, "ok");

    let response = harness.fetch("https://example.test/api/message")?;
    assert_eq!(response.url, "https://example.test/api/message");
    assert_eq!(response.status, 201);
    assert_eq!(response.body, "ok");
    assert_eq!(harness.mocks_mut().fetch().calls().len(), 1);
    assert_eq!(
        harness.mocks_mut().fetch().calls()[0].url,
        "https://example.test/api/message"
    );
    Ok(())
}

#[test]
fn missing_fetch_mock_returns_a_mock_error() -> browser_tester::Result<()> {
    let mut harness = Harness::builder().build()?;

    let error = harness
        .fetch("https://example.test/api/missing")
        .expect_err("missing fetch mock should fail");
    assert!(
        error
            .to_string()
            .contains("no fetch mock configured for `https://example.test/api/missing`")
    );
    Ok(())
}

#[test]
fn dialogs_clipboard_and_location_are_wired() -> browser_tester::Result<()> {
    let mut harness = Harness::builder().build()?;

    harness.mocks_mut().dialogs().push_confirm(true);
    harness.mocks_mut().dialogs().push_prompt(Some("Ada"));
    harness.mocks_mut().clipboard().seed_text("seeded");

    harness.alert("Notice")?;
    assert_eq!(harness.confirm("Continue?")?, true);
    assert_eq!(harness.prompt("Name?")?, Some("Ada".to_string()));
    assert_eq!(harness.read_clipboard()?, "seeded");
    harness.write_clipboard("copied")?;
    assert_eq!(harness.read_clipboard()?, "copied");
    harness.navigate("https://example.test/next")?;

    assert_eq!(
        harness.mocks_mut().dialogs().alert_messages(),
        &["Notice".to_string()]
    );
    assert_eq!(
        harness.mocks_mut().dialogs().confirm_messages(),
        &["Continue?".to_string()]
    );
    assert_eq!(
        harness.mocks_mut().dialogs().prompt_messages(),
        &["Name?".to_string()]
    );
    assert_eq!(
        harness.mocks_mut().clipboard().writes(),
        &["copied".to_string()]
    );
    assert_eq!(
        harness.mocks_mut().location().current_url(),
        Some("https://example.test/next")
    );
    assert_eq!(
        harness.mocks_mut().location().navigations(),
        &["https://example.test/next".to_string()]
    );
    assert_eq!(harness.debug().url(), "https://example.test/next");
    Ok(())
}

#[test]
fn navigator_clipboard_write_text_is_publicly_supported() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<div id='out'></div><script>window.navigator.clipboard.writeText('copied'); document.getElementById('out').textContent = String(window.navigator.clipboard) + ':' + window.navigator.clipboard.readText();</script>",
    )?;

    harness.assert_text("#out", "[object Clipboard]:copied")?;
    assert_eq!(harness.clipboard_text(), "copied");
    Ok(())
}

#[test]
fn document_location_is_wired_through_script_and_location_mock() -> browser_tester::Result<()>
{
    let mut harness = Harness::from_html(
        "<main id='out'></main><script>const beforeLocation = document.location; const beforeUrl = document.URL; document.location = 'https://example.test/next'; const afterDocumentUri = document.documentURI; const afterWindowLocation = window.location; document.getElementById('out').textContent = beforeLocation + ':' + beforeUrl + ':' + afterDocumentUri + ':' + afterWindowLocation;</script>",
    )?;

    harness.assert_text(
        "#out",
        "https://app.local/:https://app.local/:https://example.test/next:https://example.test/next",
    )?;
    assert_eq!(
        harness.mocks_mut().location().current_url(),
        Some("https://example.test/next")
    );
    assert_eq!(
        harness.mocks_mut().location().navigations(),
        &["https://example.test/next".to_string()]
    );
    assert_eq!(harness.debug().url(), "https://example.test/next");
    Ok(())
}

#[test]
fn location_assign_replace_and_reload_are_available_from_scripts() -> browser_tester::Result<()>
{
    let mut harness = Harness::from_html(
        "<main id='out'></main><script>const before = document.location; window.location.assign('https://example.test/assign'); document.location.replace('https://example.test/replace'); window.location.reload(); document.getElementById('out').textContent = before + ':' + document.location + ':' + String(window.history.length);</script>",
    )?;

    harness.assert_text("#out", "https://app.local/:https://example.test/replace:2")?;
    assert_eq!(
        harness.mocks_mut().location().current_url(),
        Some("https://example.test/replace")
    );
    assert_eq!(
        harness.mocks_mut().location().navigations(),
        &[
            "https://example.test/assign".to_string(),
            "https://example.test/replace".to_string(),
            "https://example.test/replace".to_string(),
        ]
    );
    Ok(())
}

#[test]
fn location_assign_requires_a_url_from_scripts() -> browser_tester::Result<()> {
    let error = Harness::from_html("<script>window.location.assign();</script>")
        .expect_err("location.assign should reject missing URLs");

    assert!(
        error
            .to_string()
            .contains("location.assign() expects exactly one argument")
    );
    Ok(())
}

#[test]
fn location_href_is_available_from_scripts() -> browser_tester::Result<()> {
    let mut harness = Harness::from_html(
        "<main id='out'></main><script>const before = window.location.href; document.location.href = 'https://example.test/next'; document.getElementById('out').textContent = before + ':' + window.location.href;</script>",
    )?;

    harness.assert_text("#out", "https://app.local/:https://example.test/next")?;
    assert_eq!(
        harness.mocks_mut().location().current_url(),
        Some("https://example.test/next")
    );
    assert_eq!(
        harness.mocks_mut().location().navigations(),
        &["https://example.test/next".to_string()]
    );
    Ok(())
}

#[test]
fn location_stringification_helpers_are_available_from_scripts() -> browser_tester::Result<()>
{
    let mut harness = Harness::builder()
        .url("https://app.local:8443/start?x#old")
        .html("<main id='out'></main><script>document.getElementById('out').textContent = document.location.toString() + ':' + window.location.valueOf();</script>")
        .build()?;

    harness.assert_text(
        "#out",
        "https://app.local:8443/start?x#old:https://app.local:8443/start?x#old",
    )?;
    assert_eq!(
        harness.mocks_mut().location().current_url(),
        Some("https://app.local:8443/start?x#old")
    );
    assert!(harness.mocks_mut().location().navigations().is_empty());
    Ok(())
}

#[test]
fn location_href_requires_a_url_from_scripts() -> browser_tester::Result<()> {
    let error = Harness::from_html("<script>window.location.href = '';</script>")
        .expect_err("location.href should reject missing URLs");

    assert!(
        error
            .to_string()
            .contains("navigate() requires a non-empty URL")
    );
    Ok(())
}

#[test]
fn location_hash_is_available_from_scripts() -> browser_tester::Result<()> {
    let mut harness = Harness::from_html(
        "<main id='out'></main><script>const before = window.location.hash; document.location.hash = '#next'; document.getElementById('out').textContent = before + ':' + document.location + ':' + window.location.hash;</script>",
    )?;

    harness.assert_text("#out", ":https://app.local/#next:#next")?;
    assert_eq!(
        harness.mocks_mut().location().current_url(),
        Some("https://app.local/#next")
    );
    assert_eq!(
        harness.mocks_mut().location().navigations(),
        &["https://app.local/#next".to_string()]
    );
    Ok(())
}

#[test]
fn location_pathname_is_available_from_scripts() -> browser_tester::Result<()> {
    let mut harness = Harness::builder()
        .url("https://app.local/start?x#old")
        .html("<main id='out'></main><script>const before = window.location.pathname; document.location.pathname = 'next'; document.getElementById('out').textContent = before + ':' + document.location + ':' + window.location.pathname;</script>")
        .build()?;

    harness.assert_text("#out", "/start:https://app.local/next?x#old:/next")?;
    assert_eq!(
        harness.mocks_mut().location().current_url(),
        Some("https://app.local/next?x#old")
    );
    assert_eq!(
        harness.mocks_mut().location().navigations(),
        &["https://app.local/next?x#old".to_string()]
    );
    Ok(())
}

#[test]
fn location_search_is_available_from_scripts() -> browser_tester::Result<()> {
    let mut harness = Harness::builder()
        .url("https://app.local/start?x#old")
        .html("<main id='out'></main><script>const before = window.location.search; document.location.search = '?next'; document.getElementById('out').textContent = before + ':' + document.location + ':' + window.location.search;</script>")
        .build()?;

    harness.assert_text("#out", "?x:https://app.local/start?next#old:?next")?;
    assert_eq!(
        harness.mocks_mut().location().current_url(),
        Some("https://app.local/start?next#old")
    );
    assert_eq!(
        harness.mocks_mut().location().navigations(),
        &["https://app.local/start?next#old".to_string()]
    );
    Ok(())
}

#[test]
fn location_origin_is_available_from_scripts() -> browser_tester::Result<()> {
    let mut harness = Harness::builder()
        .url("https://app.local:8443/start?x#old")
        .html("<main id='out'></main><script>const before = window.location.origin; document.location.pathname = 'next'; const after = window.location.origin; document.getElementById('out').textContent = before + ':' + after + ':' + document.location;</script>")
        .build()?;

    harness.assert_text(
        "#out",
        "https://app.local:8443:https://app.local:8443:https://app.local:8443/next?x#old",
    )?;
    assert_eq!(
        harness.mocks_mut().location().current_url(),
        Some("https://app.local:8443/next?x#old")
    );
    assert_eq!(
        harness.mocks_mut().location().navigations(),
        &["https://app.local:8443/next?x#old".to_string()]
    );
    Ok(())
}

#[test]
fn location_protocol_host_hostname_and_port_are_available_from_scripts()
-> browser_tester::Result<()> {
    let mut harness = Harness::builder()
        .url("https://app.local:8443/start?x#old")
        .html("<main id='out'></main><script>const before = window.location.protocol + '|' + window.location.host + '|' + window.location.hostname + '|' + window.location.port; document.location.protocol = 'http:'; document.location.host = 'example.test:8080'; document.location.hostname = 'example.test'; document.location.port = '8080'; const after = window.location.protocol + '|' + window.location.host + '|' + window.location.hostname + '|' + window.location.port; document.getElementById('out').textContent = before + ':' + after + ':' + document.location;</script>")
        .build()?;

    harness.assert_text(
        "#out",
        "https:|app.local:8443|app.local|8443:http:|example.test:8080|example.test|8080:http://example.test:8080/start?x#old",
    )?;
    assert_eq!(
        harness.mocks_mut().location().current_url(),
        Some("http://example.test:8080/start?x#old")
    );
    assert_eq!(
        harness.mocks_mut().location().navigations(),
        &[
            "http://app.local:8443/start?x#old".to_string(),
            "http://example.test:8080/start?x#old".to_string(),
            "http://example.test:8080/start?x#old".to_string(),
            "http://example.test:8080/start?x#old".to_string(),
        ]
    );
    Ok(())
}

#[test]
fn location_username_and_password_are_available_from_scripts() -> browser_tester::Result<()> {
    let mut harness = Harness::builder()
        .url("https://alice:secret@app.local:8443/start?x#old")
        .html("<main id='out'></main><script>const before = window.location.username + '|' + window.location.password; document.location.username = 'bob'; document.location.password = 'hunter2'; document.location.port = '9444'; const after = window.location.username + '|' + window.location.password + '|' + window.location.port; document.getElementById('out').textContent = before + ':' + after + ':' + document.location;</script>")
        .build()?;

    harness.assert_text(
        "#out",
        "alice|secret:bob|hunter2|9444:https://bob:hunter2@app.local:9444/start?x#old",
    )?;
    assert_eq!(
        harness.mocks_mut().location().current_url(),
        Some("https://bob:hunter2@app.local:9444/start?x#old")
    );
    assert_eq!(
        harness.mocks_mut().location().navigations(),
        &[
            "https://bob:secret@app.local:8443/start?x#old".to_string(),
            "https://bob:hunter2@app.local:8443/start?x#old".to_string(),
            "https://bob:hunter2@app.local:9444/start?x#old".to_string(),
        ]
    );
    Ok(())
}

#[test]
fn location_port_assignment_rejects_non_numeric_values() -> browser_tester::Result<()> {
    let error = Harness::builder()
        .url("https://app.local:8443/start?x#old")
        .html("<script>document.location.port = 'abc';</script>")
        .build()
        .expect_err("location.port should reject non-numeric values");

    assert!(
        error
            .to_string()
            .contains("unsupported location.port value: abc")
    );
    Ok(())
}

#[test]
fn document_current_script_is_available_during_inline_bootstrap() -> browser_tester::Result<()>
{
    let harness = Harness::from_html(
        "<main id='out'></main><script id='first'>document.getElementById('out').textContent = document.currentScript.getAttribute('id');</script><script id='second'>document.getElementById('out').textContent += ':' + document.currentScript.getAttribute('id');</script>",
    )?;

    harness.assert_text("#out", "first:second")?;
    Ok(())
}

#[test]
fn document_active_element_tracks_focus_state() -> browser_tester::Result<()> {
    let mut harness = Harness::from_html(
        "<input id='first'><div id='out'></div><script>document.getElementById('first').addEventListener('focus', () => { document.getElementById('out').textContent = document.activeElement.getAttribute('id'); });</script>",
    )?;

    harness.focus("#first")?;
    harness.assert_text("#out", "first")?;
    Ok(())
}

#[test]
fn document_has_focus_tracks_focus_state() -> browser_tester::Result<()> {
    let mut harness = Harness::from_html(
        "<input id='first'><div id='out'></div><script>document.getElementById('first').addEventListener('focus', () => { document.getElementById('out').textContent = String(document.hasFocus()); });</script>",
    )?;

    harness.focus("#first")?;
    harness.assert_text("#out", "true")?;
    Ok(())
}

#[test]
fn document_scrolling_element_is_publicly_supported() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<html id='html'><head id='head'><title>Title</title></head><body id='body'><main id='out'></main><script>document.getElementById('out').textContent = document.scrollingElement.getAttribute('id');</script></body></html>",
    )?;

    harness.assert_text("#out", "html")?;
    Ok(())
}

#[test]
fn document_scrolling_element_assignment_is_rejected() -> browser_tester::Result<()> {
    let error = Harness::from_html(
        "<main id='out'></main><script>document.scrollingElement = null;</script>",
    )
    .expect_err("document.scrollingElement should be read-only");

    assert!(error.to_string().contains("unsupported assignment target"));
    assert!(error.to_string().contains("scrollingElement"));
    Ok(())
}

#[test]
fn owner_document_is_publicly_supported() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<html><body id='body'><main id='out'></main><script>document.getElementById('out').textContent = String(document.body.ownerDocument) + ':' + String(document.body.ownerDocument.defaultView) + ':' + String(document.ownerDocument);</script></body></html>",
    )?;

    harness.assert_text("#out", "[object Document]:[object Window]:null")?;
    Ok(())
}

#[test]
fn parent_node_is_publicly_supported() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<html><body id='body'>Text<main id='out'></main><script>const body = document.body; const text = body.childNodes.item(0); document.getElementById('out').textContent = String(document.parentNode) + ':' + String(document.documentElement.parentNode) + ':' + String(document.documentElement.parentElement) + ':' + String(body.parentNode) + ':' + String(body.parentElement) + ':' + String(text.parentNode) + ':' + String(text.parentElement);</script></body></html>",
    )?;

    harness.assert_text(
        "#out",
        "null:[object Document]:null:[object Element]:[object Element]:[object Element]:[object Element]",
    )?;
    Ok(())
}

#[test]
fn parent_node_assignment_is_rejected() -> browser_tester::Result<()> {
    let error = Harness::from_html(
        "<body id='body'><script>document.body.parentNode = null;</script></body>",
    )
    .expect_err("parentNode should be read-only");

    assert!(error.to_string().contains("unsupported assignment target"));
    assert!(error.to_string().contains("parentNode"));
    Ok(())
}

#[test]
fn is_connected_is_publicly_supported() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<html><body><div id='ghost'></div><main id='out'></main><script>const ghost = document.getElementById('ghost'); ghost.remove(); document.getElementById('out').textContent = String(document.isConnected) + ':' + String(document.body.isConnected) + ':' + String(ghost.isConnected);</script></body></html>",
    )?;

    harness.assert_text("#out", "true:true:false")?;
    Ok(())
}

#[test]
fn is_connected_assignment_is_rejected() -> browser_tester::Result<()> {
    let error = Harness::from_html(
        "<body id='body'><script>document.body.isConnected = false;</script></body>",
    )
    .expect_err("isConnected should be read-only");

    assert!(error.to_string().contains("unsupported assignment target"));
    assert!(error.to_string().contains("isConnected"));
    Ok(())
}

#[test]
fn first_and_last_element_child_are_publicly_supported() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<html><head></head><body><div id='wrapper'><span></span><span></span></div><template id='tmpl'><span></span><span></span></template><main id='out'></main><script>const html = document.documentElement; const wrapper = document.getElementById('wrapper'); const template = document.getElementById('tmpl').content; document.getElementById('out').textContent = String(document.childElementCount) + ':' + String(document.firstElementChild) + ':' + String(document.lastElementChild) + ':' + String(html.childElementCount) + ':' + String(html.firstElementChild) + ':' + String(html.lastElementChild) + ':' + String(wrapper.childElementCount) + ':' + String(wrapper.firstElementChild) + ':' + String(wrapper.lastElementChild) + ':' + String(template.childElementCount) + ':' + String(template.firstElementChild) + ':' + String(template.lastElementChild);</script></body></html>",
    )?;

    harness.assert_text(
        "#out",
        "1:[object Element]:[object Element]:2:[object Element]:[object Element]:2:[object Element]:[object Element]:2:[object Element]:[object Element]",
    )?;
    Ok(())
}

#[test]
fn template_content_query_selector_is_publicly_supported() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<template id='tmpl'><span class='primary' id='first'>First</span><span class='primary' id='second'>Second</span></template><main id='out'></main><script>const content = document.getElementById('tmpl').content; const first = content.querySelector('.primary'); const all = content.querySelectorAll('.primary'); document.getElementById('out').textContent = String(content) + ':' + first.textContent + ':' + String(all.length) + ':' + all.item(0).textContent + ':' + all.item(1).textContent + ':' + String(all.item(2));</script>",
    )?;

    harness.assert_text(
        "#out",
        "[object DocumentFragment]:First:2:First:Second:null",
    )?;
    Ok(())
}

#[test]
fn template_content_get_element_by_id_is_publicly_supported() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<template id='tmpl'><span id='foo,bar'>First</span></template><main id='out'></main><script>const content = document.getElementById('tmpl').content; const hit = content.getElementById('foo,bar'); document.getElementById('out').textContent = String(content) + ':' + String(hit) + ':' + hit.textContent;</script>",
    )?;

    harness.assert_text("#out", "[object DocumentFragment]:[object Element]:First")?;
    Ok(())
}

#[test]
fn first_element_child_assignment_is_rejected() -> browser_tester::Result<()> {
    let error = Harness::from_html(
        "<body id='body'><script>document.body.firstElementChild = null;</script></body>",
    )
    .expect_err("firstElementChild should be read-only");

    assert!(error.to_string().contains("unsupported assignment target"));
    assert!(error.to_string().contains("firstElementChild"));
    Ok(())
}

#[test]
fn owner_document_assignment_is_rejected() -> browser_tester::Result<()> {
    let error = Harness::from_html(
        "<body id='body'><script>document.body.ownerDocument = null;</script></body>",
    )
    .expect_err("ownerDocument should be read-only");

    assert!(error.to_string().contains("unsupported assignment target"));
    assert!(error.to_string().contains("ownerDocument"));
    Ok(())
}

#[test]
fn window_children_are_publicly_supported() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<div id='root'><span id='first'>First</span><span id='second'>Second</span></div><div id='out'></div><script>const children = document.defaultView.children; document.getElementById('out').textContent = String(children.length) + ':' + children.item(0).textContent + ':' + children.item(1).textContent + ':' + String(children.namedItem('first')) + ':' + String(children.namedItem('missing'));</script>",
    )?;

    harness.assert_text("#out", "3:FirstSecond::null:null")?;
    Ok(())
}

#[test]
fn window_frames_iterator_helpers_are_publicly_supported() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<iframe id='first'></iframe><iframe id='second'></iframe><div id='out'></div><script>const frames = document.defaultView.frames; const keys = frames.keys(); const values = frames.values(); const entries = frames.entries(); const firstKey = keys.next(); const firstValue = values.next(); const firstEntry = entries.next(); let out = ''; frames.forEach((element, index, list) => { out += String(index) + ':' + element.getAttribute('id') + ':' + String(list.length) + ';'; }); document.getElementById('out').textContent = String(firstKey.value) + ':' + firstValue.value.getAttribute('id') + ':' + String(firstEntry.value.index) + ':' + firstEntry.value.value.getAttribute('id') + ':' + out + String(frames.length);</script>",
    )?;

    harness.assert_text("#out", "0:first:0:first:0:first:2;1:second:2;2")?;
    Ok(())
}

#[test]
fn html_collection_named_property_access_is_publicly_supported() -> browser_tester::Result<()>
{
    let harness = Harness::from_html(
        "<div id='root'><span id='first'>First</span><form id='signup'><input type='radio' name='mode' id='mode-a' value='a'><input type='radio' name='mode' id='mode-b' value='b'></form></div><div id='out'></div><script>const children = document.getElementById('root').children; const mode = document.getElementById('signup').elements.mode; document.getElementById('out').textContent = children.first.textContent + ':' + String(children.missing) + ':' + String(mode.length) + ':' + mode.item(0).value + ':' + mode.item(1).value + ':' + String(mode);</script>",
    )?;

    harness.assert_text("#out", "First:undefined:2:a:b:[object RadioNodeList]")?;
    Ok(())
}

#[test]
fn radio_node_list_iterator_helpers_are_publicly_supported() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<div id='root'><form id='signup'><input type='radio' name='mode' id='mode-a' value='a'><input type='radio' name='mode' id='mode-b' value='b'></form><div id='out'></div><script>const elements = document.getElementById('signup').elements; const named = elements.namedItem('mode'); const keys = named.keys(); const values = named.values(); const entries = named.entries(); const firstKey = keys.next(); const firstValue = values.next(); const firstEntry = entries.next(); let out = ''; named.forEach((element, index, list) => { out += String(index) + ':' + element.value + ':' + String(list.length) + ';'; }); document.getElementById('out').textContent = String(named.length) + ':' + String(firstKey.value) + ':' + firstValue.value.value + ':' + String(firstEntry.value.index) + ':' + firstEntry.value.value.value + ':' + out;</script></div>",
    )?;

    harness.assert_text("#out", "2:0:a:0:a:0:a:2;1:b:2;")?;
    Ok(())
}

#[test]
fn window_frames_are_publicly_supported() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<iframe id='first' name='first'></iframe><iframe id='second'></iframe><div id='out'></div><script>const frames = window.frames; const before = frames.length; document.getElementById('second').remove(); document.getElementById('out').textContent = String(before) + ':' + String(frames.length) + ':' + frames.item(0).getAttribute('id') + ':' + frames.namedItem('first').getAttribute('id') + ':' + String(frames.namedItem('missing'));</script>",
    )?;

    harness.assert_text("#out", "2:1:first:first:null")?;
    Ok(())
}

#[test]
fn document_forms_iterator_helpers_are_publicly_supported() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<div id='root'><form id='first-form' name='first-form'>First</form><form id='second-form' name='second-form'>Second</form></div><div id='out'></div><script>const forms = document.forms; const keys = forms.keys(); const values = forms.values(); const entries = forms.entries(); const firstKey = keys.next(); const firstValue = values.next(); const firstEntry = entries.next(); let out = ''; forms.forEach((element, index, list) => { out += String(index) + ':' + element.getAttribute('id') + ':' + String(list.length) + ';'; }); document.getElementById('out').textContent = String(firstKey.value) + ':' + firstValue.value.getAttribute('id') + ':' + String(firstEntry.value.index) + ':' + firstEntry.value.value.getAttribute('id') + ':' + out;</script>",
    )?;

    harness.assert_text(
        "#out",
        "0:first-form:0:first-form:0:first-form:2;1:second-form:2;",
    )?;
    Ok(())
}

#[test]
fn window_frames_length_assignment_is_rejected() -> browser_tester::Result<()> {
    let error = Harness::from_html(
        "<iframe id='first'></iframe><script>window.frames.length = 2;</script>",
    )
    .expect_err("window.frames.length should be read-only");

    assert!(
        error
            .to_string()
            .contains("cannot assign to `length` on html collection value")
    );
    Ok(())
}

#[test]
fn window_frame_element_is_publicly_supported() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<div id='out'></div><script>document.getElementById('out').textContent = String(window.frameElement);</script>",
    )?;

    harness.assert_text("#out", "null")?;
    Ok(())
}

#[test]
fn window_frame_element_assignment_is_rejected() -> browser_tester::Result<()> {
    let error = Harness::from_html("<script>window.frameElement = 2;</script>")
        .expect_err("window.frameElement should be read-only");

    assert!(error.to_string().contains("unsupported assignment target"));
    Ok(())
}

#[test]
fn window_opener_is_publicly_supported() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<div id='out'></div><script>document.getElementById('out').textContent = String(window.opener) + ':' + String(document.defaultView.opener);</script>",
    )?;

    harness.assert_text("#out", "null:null")?;
    Ok(())
}

#[test]
fn window_opener_assignment_is_rejected() -> browser_tester::Result<()> {
    let error = Harness::from_html("<script>window.opener = 2;</script>")
        .expect_err("window.opener should be read-only");

    assert!(error.to_string().contains("unsupported assignment target"));
    Ok(())
}

#[test]
fn form_and_select_lengths_are_publicly_supported() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='out'></main><form id='signup'><input><input></form><select id='mode'><option>A</option><option>B</option></select><script>document.getElementById('out').textContent = String(document.getElementById('signup').length) + ':' + String(document.getElementById('mode').length);</script>",
    )?;

    harness.assert_text("#out", "2:2")?;
    Ok(())
}

#[test]
fn form_length_assignment_is_rejected() -> browser_tester::Result<()> {
    let error = Harness::from_html("<form id='signup'><input><input></form><script>document.getElementById('signup').length = 2;</script>")
        .expect_err("form.length should be read-only");

    assert!(error.to_string().contains("unsupported assignment target"));
    Ok(())
}

#[test]
fn window_length_is_publicly_supported() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<iframe id='first' name='first'></iframe><iframe id='second'></iframe><div id='out'></div><script>const before = window.length; document.getElementById('second').remove(); document.getElementById('out').textContent = String(before) + ':' + String(window.length);</script>",
    )?;

    harness.assert_text("#out", "2:1")?;
    Ok(())
}

#[test]
fn window_length_assignment_is_rejected() -> browser_tester::Result<()> {
    let error =
        Harness::from_html("<iframe id='first'></iframe><script>window.length = 2;</script>")
            .expect_err("window.length should be read-only");

    assert!(error.to_string().contains("unsupported assignment target"));
    Ok(())
}

#[test]
fn document_ready_state_is_loading_during_inline_bootstrap() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='out'></main><script>document.getElementById('out').textContent = document.readyState;</script>",
    )?;

    harness.assert_text("#out", "loading")?;
    Ok(())
}

#[test]
fn document_compat_mode_is_publicly_supported() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='out'></main><script>document.getElementById('out').textContent = document.compatMode;</script>",
    )?;

    harness.assert_text("#out", "CSS1Compat")?;
    Ok(())
}

#[test]
fn document_character_set_is_publicly_supported() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='out'></main><script>document.getElementById('out').textContent = document.characterSet + ':' + document.charset;</script>",
    )?;

    harness.assert_text("#out", "UTF-8:UTF-8")?;
    Ok(())
}

#[test]
fn document_content_type_is_publicly_supported() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='out'></main><script>document.getElementById('out').textContent = document.contentType;</script>",
    )?;

    harness.assert_text("#out", "text/html")?;
    Ok(())
}

#[test]
fn document_design_mode_is_publicly_supported() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='out'></main><script>const before = document.designMode; document.designMode = 'on'; document.getElementById('out').textContent = before + ':' + document.designMode;</script>",
    )?;

    harness.assert_text("#out", "off:on")?;
    Ok(())
}

#[test]
fn document_design_mode_rejects_invalid_values() -> browser_tester::Result<()> {
    let error = Harness::from_html("<script>document.designMode = 'maybe';</script>")
        .expect_err("document.designMode should reject unsupported values");

    assert!(error.to_string().contains("designMode"));
    assert!(error.to_string().contains("unsupported"));
    Ok(())
}

#[test]
fn element_content_editable_is_publicly_supported() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='out'></main><div id='parent' contenteditable='true'><span id='child' contenteditable='false'>Edit</span></div><script>const child = document.getElementById('child'); const before = child.contentEditable; const beforeState = child.isContentEditable; child.contentEditable = 'inherit'; const after = child.contentEditable; const afterState = child.isContentEditable; document.getElementById('out').textContent = before + ':' + String(beforeState) + ':' + after + ':' + String(afterState) + ':' + String(child.matches(':read-write'));</script>",
    )?;

    harness.assert_text("#out", "false:false:inherit:true:true")?;
    Ok(())
}

#[test]
fn element_content_editable_rejects_invalid_values() -> browser_tester::Result<()> {
    let error = Harness::from_html(
        "<div id='editable' contenteditable='true'></div><script>document.getElementById('editable').contentEditable = 'maybe';</script>",
    )
    .expect_err("element.contentEditable should reject unsupported values");

    assert!(error.to_string().contains("contentEditable"));
    assert!(error.to_string().contains("unsupported"));
    Ok(())
}

#[test]
fn document_visibility_state_and_hidden_are_publicly_supported() -> browser_tester::Result<()>
{
    let harness = Harness::from_html(
        "<main id='out'></main><script>document.getElementById('out').textContent = document.visibilityState + ':' + String(document.hidden);</script>",
    )?;

    harness.assert_text("#out", "visible:false")?;
    Ok(())
}

#[test]
fn window_device_pixel_ratio_is_publicly_supported() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='out'></main><script>document.getElementById('out').textContent = String(window.devicePixelRatio);</script>",
    )?;

    harness.assert_text("#out", "1")?;
    Ok(())
}

#[test]
fn window_inner_width_and_inner_height_are_publicly_supported() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='out'></main><script>document.getElementById('out').textContent = String(window.innerWidth) + ':' + String(window.innerHeight);</script>",
    )?;

    harness.assert_text("#out", "1024:768")?;
    Ok(())
}

#[test]
fn window_outer_width_and_outer_height_are_publicly_supported() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='out'></main><script>document.getElementById('out').textContent = String(window.outerWidth) + ':' + String(window.outerHeight);</script>",
    )?;

    harness.assert_text("#out", "1024:768")?;
    Ok(())
}

#[test]
fn window_screen_position_is_publicly_supported() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='out'></main><script>document.getElementById('out').textContent = String(window.screenX) + ':' + String(window.screenY) + ':' + String(window.screenLeft) + ':' + String(window.screenTop);</script>",
    )?;

    harness.assert_text("#out", "0:0:0:0")?;
    Ok(())
}

#[test]
fn window_screen_object_is_publicly_supported() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='out'></main><script>document.getElementById('out').textContent = String(window.screen) + ':' + String(window.screen.width) + ':' + String(window.screen.height) + ':' + String(window.screen.availWidth) + ':' + String(window.screen.availHeight) + ':' + String(window.screen.availLeft) + ':' + String(window.screen.availTop) + ':' + String(window.screen.colorDepth) + ':' + String(window.screen.pixelDepth);</script>",
    )?;

    harness.assert_text("#out", "[object Screen]:1024:768:1024:768:0:0:24:24")?;
    Ok(())
}

#[test]
fn window_screen_orientation_is_publicly_supported() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='out'></main><script>document.getElementById('out').textContent = String(window.screen.orientation) + ':' + window.screen.orientation.type + ':' + String(window.screen.orientation.angle);</script>",
    )?;

    harness.assert_text("#out", "[object ScreenOrientation]:landscape-primary:0")?;
    Ok(())
}

#[test]
fn window_screen_orientation_assignment_is_rejected() -> browser_tester::Result<()> {
    let error = Harness::from_html(
        "<main id='out'></main><script>window.screen.orientation.type = 'portrait-primary';</script>",
    )
    .expect_err("window.screen.orientation.type should be rejected");

    assert!(error.to_string().contains("screen orientation"));
    assert!(error.to_string().contains("type"));
    Ok(())
}

#[test]
fn document_referrer_is_publicly_supported() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='out'></main><script>document.getElementById('out').textContent = '[' + document.referrer + ']';</script>",
    )?;

    harness.assert_text("#out", "[]")?;
    Ok(())
}

#[test]
fn window_name_is_publicly_supported() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='out'></main><script>const before = window.name; window.self.name = 'updated'; document.getElementById('out').textContent = before + ':' + window.window.name + ':' + window.parent.name + ':' + window.top.name;</script>",
    )?;

    harness.assert_text("#out", ":updated:updated:updated")?;
    Ok(())
}

#[test]
fn window_self_assignment_is_rejected() -> browser_tester::Result<()> {
    let error =
        Harness::from_html("<main id='out'></main><script>window.self = 'updated';</script>")
            .expect_err("window.self should be read-only");

    assert!(error.to_string().contains("unsupported assignment target"));
    assert!(error.to_string().contains("self"));
    Ok(())
}

#[test]
fn window_closed_is_publicly_supported() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='out'></main><script>document.getElementById('out').textContent = String(window.closed) + ':' + String(window.self.closed) + ':' + String(window.window.closed) + ':' + String(window.parent.closed) + ':' + String(window.top.closed);</script>",
    )?;

    harness.assert_text("#out", "false:false:false:false:false")?;
    Ok(())
}

#[test]
fn window_closed_assignment_is_rejected() -> browser_tester::Result<()> {
    let error = Harness::from_html("<main id='out'></main><script>window.closed = true;</script>")
        .expect_err("window.closed should be read-only");

    assert!(error.to_string().contains("unsupported assignment target"));
    assert!(error.to_string().contains("closed"));
    Ok(())
}

#[test]
fn window_history_is_publicly_supported() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='out'></main><script>document.getElementById('out').textContent = String(window.history) + ':' + String(window.history.length) + ':' + String(window.self.history.length) + ':' + String(window.history.state) + ':' + String(window.history.scrollRestoration);</script>",
    )?;

    harness.assert_text("#out", "[object History]:1:1:null:auto")?;
    Ok(())
}

#[test]
fn window_history_push_and_replace_state_are_publicly_supported() -> browser_tester::Result<()>
{
    let harness = Harness::from_html(
        "<main id='out'></main><script>window.history.pushState('step-1', '', 'https://app.local/step-1'); window.history.replaceState('step-2', '', 'https://app.local/step-2'); document.getElementById('out').textContent = document.location + ':' + String(window.history.length) + ':' + String(window.history.state);</script>",
    )?;

    harness.assert_text("#out", "https://app.local/step-2:2:step-2")?;
    Ok(())
}

#[test]
fn window_history_assignment_is_rejected() -> browser_tester::Result<()> {
    let error =
        Harness::from_html("<main id='out'></main><script>window.history.length = 2;</script>")
            .expect_err("window.history should be read-only");

    assert!(error.to_string().contains("history"));
    assert!(error.to_string().contains("length"));
    Ok(())
}

#[test]
fn window_history_state_assignment_is_rejected() -> browser_tester::Result<()> {
    let error =
        Harness::from_html("<main id='out'></main><script>window.history.state = 'step';</script>")
            .expect_err("window.history.state should be read-only");

    assert!(error.to_string().contains("history"));
    assert!(error.to_string().contains("state"));
    Ok(())
}

#[test]
fn window_history_push_state_arity_is_rejected() -> browser_tester::Result<()> {
    let error = Harness::from_html(
        "<main id='out'></main><script>window.history.pushState('step');</script>",
    )
    .expect_err("window.history.pushState should reject too few arguments");

    assert!(
        error
            .to_string()
            .contains("history.pushState() expects 2 or 3 arguments")
    );
    Ok(())
}

#[test]
fn window_history_replace_state_arity_is_rejected() -> browser_tester::Result<()> {
    let error = Harness::from_html(
        "<main id='out'></main><script>window.history.replaceState('step');</script>",
    )
    .expect_err("window.history.replaceState should reject too few arguments");

    assert!(
        error
            .to_string()
            .contains("history.replaceState() expects 2 or 3 arguments")
    );
    Ok(())
}

#[test]
fn window_history_scroll_restoration_is_publicly_supported() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='out'></main><script>window.history.scrollRestoration = 'manual'; document.getElementById('out').textContent = String(window.history.scrollRestoration);</script>",
    )?;

    harness.assert_text("#out", "manual")?;
    Ok(())
}

#[test]
fn window_history_scroll_restoration_assignment_is_rejected() -> browser_tester::Result<()> {
    let error = Harness::from_html(
        "<main id='out'></main><script>window.history.scrollRestoration = 'sideways';</script>",
    )
    .expect_err("window.history.scrollRestoration should reject invalid values");

    assert!(error.to_string().contains("scroll restoration"));
    Ok(())
}

#[test]
fn window_history_methods_are_publicly_supported() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='out'></main><script>window.location = 'https://app.local/step-1'; window.location = 'https://app.local/step-2'; window.history.back(); window.history.forward(); window.history.go(-1); document.getElementById('out').textContent = document.location + ':' + String(window.history.length);</script>",
    )?;

    harness.assert_text("#out", "https://app.local/step-1:3")?;
    Ok(())
}

#[test]
fn window_history_back_arguments_are_rejected() -> browser_tester::Result<()> {
    let error =
        Harness::from_html("<main id='out'></main><script>window.history.back(1);</script>")
            .expect_err("window.history.back should reject arguments");

    assert!(
        error
            .to_string()
            .contains("history.back() expects no arguments")
    );
    Ok(())
}

#[test]
fn document_dir_is_publicly_supported() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='root' dir='ltr'><main id='out'></main><script>const before = document.dir; document.dir = 'rtl'; document.getElementById('out').textContent = before + ':' + document.dir + ':' + document.documentElement.getAttribute('dir');</script></main>",
    )?;

    harness.assert_text("#out", "ltr:rtl:rtl")?;
    Ok(())
}

#[test]
fn document_url_assignment_is_rejected() -> browser_tester::Result<()> {
    let error = Harness::from_html(
        "<main id='out'></main><script>document.URL = 'https://example.test/next';</script>",
    )
    .expect_err("document.URL should be read-only");

    assert!(error.to_string().contains("unsupported assignment target"));
    assert!(error.to_string().contains("URL"));
    Ok(())
}

#[test]
fn document_base_uri_aliases_are_publicly_supported() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><span id='child'></span></main><div id='out'></div><script>const root = document.getElementById('root'); const child = document.getElementById('child'); document.getElementById('out').textContent = document.baseURI + ':' + root.baseURI + ':' + child.baseURI + ':' + document.documentURI;</script>",
    )?;

    harness.assert_text(
        "#out",
        "https://app.local/:https://app.local/:https://app.local/:https://app.local/",
    )?;
    Ok(())
}

#[test]
fn document_origin_aliases_are_publicly_supported() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><span id='child'></span></main><div id='out'></div><script>const root = document.getElementById('root'); const child = document.getElementById('child'); document.getElementById('out').textContent = document.domain + ':' + document.origin + ':' + window.origin + ':' + root.origin + ':' + child.origin;</script>",
    )?;

    harness.assert_text(
        "#out",
        "app.local:https://app.local:https://app.local:https://app.local:https://app.local",
    )?;
    Ok(())
}

#[test]
fn document_domain_assignment_is_rejected() -> browser_tester::Result<()> {
    let error =
        Harness::from_html("<main id='out'></main><script>document.domain = 'app.local';</script>")
            .expect_err("document.domain should be read-only");

    assert!(error.to_string().contains("unsupported assignment target"));
    assert!(error.to_string().contains("domain"));
    Ok(())
}

#[test]
fn web_storage_is_publicly_supported() -> browser_tester::Result<()> {
    let harness = Harness::from_html_with_local_storage(
        "<main id='out'></main><script>const local = window.localStorage; const session = document.defaultView.sessionStorage; const before = String(local) + ':' + String(session) + ':' + String(local.length) + ':' + String(session.length); const token = local.token; local.theme = 'dark'; local.removeItem('token'); session.scratch = 'xyz'; const sessionNamed = session.scratch; const sessionKey = session.key(0); session.clear(); document.getElementById('out').textContent = before + '|' + token + ':' + local.theme + ':' + sessionNamed + ':' + String(local.length) + ':' + String(local.key(0)) + ':' + String(session.length) + ':' + String(sessionKey);</script>",
        [("token", "abc")],
    )?;

    harness.assert_text(
        "#out",
        "[object Storage]:[object Storage]:1:0|abc:dark:xyz:1:theme:0:scratch",
    )?;
    Ok(())
}

#[test]
fn match_media_is_publicly_supported() -> browser_tester::Result<()> {
    let mut harness = Harness::builder()
        .html("<main id='out'></main><script>const list = window.matchMedia('(prefers-color-scheme: dark)'); document.getElementById('out').textContent = String(list.matches) + ':' + list.media + ':' + String(window.matchMedia('(prefers-color-scheme: dark)'));</script>")
        .match_media([("(prefers-color-scheme: dark)", true)])
        .build()?;

    harness.assert_text(
        "#out",
        "true:(prefers-color-scheme: dark):[object MediaQueryList]",
    )?;
    assert_eq!(
        harness.mocks_mut().match_media().calls(),
        &[
            browser_tester::MatchMediaCall {
                query: "(prefers-color-scheme: dark)".to_string(),
            },
            browser_tester::MatchMediaCall {
                query: "(prefers-color-scheme: dark)".to_string(),
            }
        ]
    );
    Ok(())
}

#[test]
fn match_media_listeners_are_publicly_supported() -> browser_tester::Result<()> {
    let mut harness = Harness::builder()
        .html("<main id='out'></main><script>const out = document.getElementById('out'); out.textContent = 'before'; const list = window.matchMedia('(prefers-color-scheme: dark)'); list.addListener(() => { out.textContent = 'called'; }); list.removeListener(() => { out.textContent = 'removed'; }); out.textContent += ':' + String(list.matches) + ':' + list.media + ':' + String(window.matchMedia('(prefers-color-scheme: dark)'));</script>")
        .match_media([("(prefers-color-scheme: dark)", true)])
        .build()?;

    harness.assert_text(
        "#out",
        "before:true:(prefers-color-scheme: dark):[object MediaQueryList]",
    )?;
    assert_eq!(
        harness.mocks_mut().match_media().calls(),
        &[
            browser_tester::MatchMediaCall {
                query: "(prefers-color-scheme: dark)".to_string(),
            },
            browser_tester::MatchMediaCall {
                query: "(prefers-color-scheme: dark)".to_string(),
            }
        ]
    );
    assert_eq!(
        harness.mocks_mut().match_media().listener_calls(),
        &[
            browser_tester::MatchMediaListenerCall {
                query: "(prefers-color-scheme: dark)".to_string(),
                method: "addListener".to_string(),
            },
            browser_tester::MatchMediaListenerCall {
                query: "(prefers-color-scheme: dark)".to_string(),
                method: "removeListener".to_string(),
            },
        ]
    );
    Ok(())
}

#[test]
fn print_is_publicly_supported_and_captured() -> browser_tester::Result<()> {
    let mut harness = Harness::from_html(
        "<main id='out'></main><script>window.print(); document.getElementById('out').textContent = 'done';</script>",
    )?;

    harness.print()?;
    assert_eq!(harness.mocks_mut().print().calls().len(), 2);
    harness.assert_text("#out", "done")?;
    Ok(())
}

#[test]
fn navigator_metadata_is_publicly_supported() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='out'></main><script>const languages = window.navigator.languages; document.getElementById('out').textContent = window.navigator.userAgent + ':' + window.navigator.appCodeName + ':' + window.navigator.productSub + ':' + window.navigator.vendor + ':' + window.navigator.vendorSub + ':' + String(window.navigator.pdfViewerEnabled) + ':' + window.navigator.doNotTrack + ':' + String(window.navigator.javaEnabled()) + ':' + window.navigator.platform + ':' + window.navigator.language + ':' + window.navigator.userLanguage + ':' + window.navigator.browserLanguage + ':' + window.navigator.systemLanguage + ':' + window.navigator.oscpu + ':' + String(languages.length) + ':' + languages.item(0) + ':' + languages.toString() + ':' + String(languages.contains('en-US')) + ':' + String(languages.contains('fr-FR')) + ':' + String(window.navigator.cookieEnabled) + ':' + String(window.navigator.onLine) + ':' + String(window.navigator.webdriver);</script>",
    )?;

    harness.assert_text(
        "#out",
        "browser_tester:browser_tester:browser_tester:browser_tester:browser_tester:false:unspecified:false:unknown:en-US:en-US:en-US:en-US:unknown:1:en-US:[object DOMStringList]:true:false:true:true:false",
    )?;
    Ok(())
}

#[test]
fn navigator_languages_iterator_helpers_are_publicly_supported() -> browser_tester::Result<()>
{
    let harness = Harness::from_html(
        "<div id='out'></div><script>const languages = window.navigator.languages; const keys = languages.keys(); const values = languages.values(); const entries = languages.entries(); const firstKey = keys.next(); const firstValue = values.next(); const firstEntry = entries.next(); const secondKey = keys.next(); const secondValue = values.next(); const secondEntry = entries.next(); document.getElementById('out').textContent = String(firstKey.value) + ':' + String(firstValue.value) + ':' + String(firstEntry.value.index) + ':' + firstEntry.value.value + ':' + String(secondKey.done) + ':' + String(secondValue.done) + ':' + String(secondEntry.done);</script>",
    )?;

    harness.assert_text("#out", "0:en-US:0:en-US:true:true:true")?;
    Ok(())
}

#[test]
fn navigator_plugins_is_publicly_supported() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<div id='root'><embed id='first-embed'><embed name='second-embed'></div><div id='out'></div><script>document.getElementById('out').textContent = String(window.navigator.plugins.length) + ':' + String(window.navigator.plugins.namedItem('first-embed')) + ':' + String(window.navigator.plugins.namedItem('missing'));</script>",
    )?;

    harness.assert_text("#out", "2:[object Element]:null")?;
    Ok(())
}

#[test]
fn navigator_plugins_iterator_helpers_are_publicly_supported() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<div id='root'><embed id='first-embed'><embed id='second-embed'></div><div id='out'></div><script>const plugins = window.navigator.plugins; const keys = plugins.keys(); const values = plugins.values(); const entries = plugins.entries(); const firstKey = keys.next(); const firstValue = values.next(); const firstEntry = entries.next(); let out = ''; plugins.forEach((element, index, list) => { out += String(index) + ':' + element.getAttribute('id') + ':' + String(list.length) + ';'; }); document.getElementById('out').textContent = String(firstKey.value) + ':' + firstValue.value.getAttribute('id') + ':' + String(firstEntry.value.index) + ':' + firstEntry.value.value.getAttribute('id') + ':' + out;</script>",
    )?;

    harness.assert_text(
        "#out",
        "0:first-embed:0:first-embed:0:first-embed:2;1:second-embed:2;",
    )?;
    Ok(())
}

#[test]
fn navigator_plugins_refresh_is_publicly_supported() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<div id='out'></div><script>document.getElementById('out').textContent = String(document.plugins.refresh()) + ':' + String(window.navigator.plugins.refresh());</script>",
    )?;

    harness.assert_text("#out", "undefined:undefined")?;
    Ok(())
}

#[test]
fn navigator_mime_types_is_publicly_supported() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<div id='out'></div><script>document.getElementById('out').textContent = String(window.navigator.mimeTypes.length) + ':' + String(window.navigator.mimeTypes.item(0)) + ':' + String(window.navigator.mimeTypes.namedItem('missing'));</script>",
    )?;

    harness.assert_text("#out", "0:null:null")?;
    Ok(())
}

#[test]
fn navigator_mime_types_iterator_helpers_are_publicly_supported() -> browser_tester::Result<()>
{
    let harness = Harness::from_html(
        "<div id='out'></div><script>const mimeTypes = window.navigator.mimeTypes; document.getElementById('out').textContent = String(mimeTypes.keys().next().done) + ':' + String(mimeTypes.values().next().done) + ':' + String(mimeTypes.entries().next().done);</script>",
    )?;

    harness.assert_text("#out", "true:true:true")?;
    Ok(())
}

#[test]
fn collection_to_string_helpers_are_publicly_supported() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='out'></main><script>document.getElementById('out').textContent = document.childNodes.toString() + ':' + document.plugins.toString() + ':' + window.navigator.mimeTypes.toString();</script>",
    )?;

    harness.assert_text(
        "#out",
        "[object NodeList]:[object HTMLCollection]:[object MimeTypeArray]",
    )?;
    Ok(())
}

#[test]
fn scroll_is_publicly_supported_and_captured() -> browser_tester::Result<()> {
    let mut harness = Harness::from_html(
        "<main id='out'></main><script>window.scrollTo(10, 20); document.getElementById('out').textContent = 'done';</script>",
    )?;

    harness.scroll_by(-5, 3)?;
    assert_eq!(harness.mocks_mut().scroll().calls().len(), 2);
    assert_eq!(
        harness.mocks_mut().scroll().calls()[0],
        browser_tester::ScrollCall {
            method: browser_tester::ScrollMethod::To,
            x: 10,
            y: 20,
        }
    );
    assert_eq!(
        harness.mocks_mut().scroll().calls()[1],
        browser_tester::ScrollCall {
            method: browser_tester::ScrollMethod::By,
            x: -5,
            y: 3,
        }
    );
    harness.assert_text("#out", "done")?;
    Ok(())
}

#[test]
fn scroll_position_aliases_are_publicly_supported() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='out'></main><script>window.scrollTo(10, 20); window.scrollBy(-5, 3); document.getElementById('out').textContent = String(window.scrollX) + ':' + String(window.scrollY) + ':' + String(window.pageXOffset) + ':' + String(window.pageYOffset);</script>",
    )?;

    harness.assert_text("#out", "5:23:5:23")?;
    Ok(())
}

#[test]
fn close_is_publicly_supported_and_captured() -> browser_tester::Result<()> {
    let mut harness = Harness::from_html(
        "<main id='out'></main><script>window.close(); document.getElementById('out').textContent = 'done';</script>",
    )?;

    harness.close()?;
    assert_eq!(harness.mocks_mut().close().calls().len(), 2);
    harness.assert_text("#out", "done")?;
    Ok(())
}

#[test]
fn open_is_publicly_supported_and_captured() -> browser_tester::Result<()> {
    let mut harness = Harness::from_html(
        "<main id='out'></main><script>window.open('https://example.test/popup', '_blank', 'noopener'); document.getElementById('out').textContent = 'done';</script>",
    )?;

    harness.open("https://example.test/direct")?;
    assert_eq!(harness.mocks_mut().open().calls().len(), 2);
    assert_eq!(
        harness.mocks_mut().open().calls()[0].url.as_deref(),
        Some("https://example.test/popup")
    );
    assert_eq!(
        harness.mocks_mut().open().calls()[0].target.as_deref(),
        Some("_blank")
    );
    assert_eq!(
        harness.mocks_mut().open().calls()[0].features.as_deref(),
        Some("noopener")
    );
    assert_eq!(
        harness.mocks_mut().open().calls()[1].url.as_deref(),
        Some("https://example.test/direct")
    );
    harness.assert_text("#out", "done")?;
    Ok(())
}

#[test]
fn scroll_failure_can_be_seeded_from_the_builder() -> browser_tester::Result<()> {
    let error = Harness::builder()
        .scroll_failure("scroll blocked")
        .html("<main id='out'></main><script>window.scrollTo(10, 20);</script>")
        .build()
        .expect_err("scroll failures should fail bootstrap when window.scrollTo runs");

    assert!(error.to_string().contains("scroll blocked"));
    Ok(())
}

#[test]
fn close_failure_can_be_seeded_from_the_builder() -> browser_tester::Result<()> {
    let error = Harness::builder()
        .close_failure("window closed")
        .html("<main id='out'></main><script>window.close();</script>")
        .build()
        .expect_err("close failures should fail bootstrap when window.close runs");

    assert!(error.to_string().contains("window closed"));
    Ok(())
}

#[test]
fn print_failure_can_be_seeded_from_the_builder() -> browser_tester::Result<()> {
    let error = Harness::builder()
        .print_failure("print blocked")
        .html("<main id='out'></main><script>window.print();</script>")
        .build()
        .expect_err("print failures should fail bootstrap when print is called");

    assert!(error.to_string().contains("print blocked"));
    Ok(())
}

#[test]
fn open_failure_can_be_seeded_from_the_builder() -> browser_tester::Result<()> {
    let error = Harness::builder()
        .open_failure("popup blocked")
        .html("<main id='out'></main><script>window.open('https://example.test/popup');</script>")
        .build()
        .expect_err("open failures should fail bootstrap when window.open runs");

    assert!(error.to_string().contains("popup blocked"));
    Ok(())
}

#[test]
fn window_alert_is_publicly_supported_from_scripts() -> browser_tester::Result<()> {
    let mut harness = Harness::from_html("<script>window.alert('Notice');</script>")?;

    assert_eq!(
        harness.mocks_mut().dialogs().alert_messages(),
        &["Notice".to_string()]
    );
    Ok(())
}

#[test]
fn window_confirm_requires_seeded_response_from_scripts() -> browser_tester::Result<()> {
    let error = Harness::from_html("<script>window.confirm('Continue?');</script>")
        .expect_err("window.confirm should require a queued response");

    assert!(
        error
            .to_string()
            .contains("confirm() requires a queued response")
    );
    Ok(())
}

#[test]
fn window_prompt_requires_seeded_response_from_scripts() -> browser_tester::Result<()> {
    let error = Harness::from_html("<script>window.prompt('Name?');</script>")
        .expect_err("window.prompt should require a queued response");

    assert!(
        error
            .to_string()
            .contains("prompt() requires a queued response")
    );
    Ok(())
}

#[test]
fn dialogs_and_clipboard_reads_require_seeded_values() -> browser_tester::Result<()> {
    let mut harness = Harness::builder().build()?;

    let confirm_error = harness
        .confirm("Continue?")
        .expect_err("confirm should require a queued response");
    assert!(
        confirm_error
            .to_string()
            .contains("confirm() requires a queued response")
    );

    let prompt_error = harness
        .prompt("Name?")
        .expect_err("prompt should require a queued response");
    assert!(
        prompt_error
            .to_string()
            .contains("prompt() requires a queued response")
    );

    let clipboard_error = harness
        .read_clipboard()
        .expect_err("clipboard reads should require a seed");
    assert!(
        clipboard_error
            .to_string()
            .contains("clipboard text has not been seeded")
    );
    Ok(())
}

#[test]
fn download_capture_is_publicly_wired() -> browser_tester::Result<()> {
    let mut harness = Harness::builder().build()?;

    harness.capture_download("report.csv", b"downloaded bytes".to_vec())?;

    {
        let mut mocks = harness.mocks_mut();
        let downloads = mocks.downloads();
        assert_eq!(downloads.artifacts().len(), 1);
        assert_eq!(downloads.artifacts()[0].file_name, "report.csv");
        assert_eq!(downloads.artifacts()[0].bytes, b"downloaded bytes".to_vec());
    }
    Ok(())
}

#[test]
fn capture_download_rejects_blank_file_names() -> browser_tester::Result<()> {
    let mut harness = Harness::builder().build()?;

    let error = harness
        .capture_download("   ", b"downloaded bytes".to_vec())
        .expect_err("blank download names should fail");
    assert!(
        error
            .to_string()
            .contains("capture_download() requires a non-empty file name")
    );
    Ok(())
}

#[test]
fn file_input_selection_updates_dom_and_capture() -> browser_tester::Result<()> {
    let mut harness = Harness::from_html(
        "<input id='upload' type='file'><div id='out'></div><script>document.getElementById('upload').addEventListener('change', () => { document.getElementById('out').textContent = document.getElementById('upload').value; });</script>",
    )?;

    harness.set_files("#upload", ["report.csv"])?;

    harness.assert_value("#upload", "report.csv")?;
    harness.assert_text("#out", "report.csv")?;
    assert_eq!(
        harness.mocks_mut().file_input().selections()[0].selector,
        "#upload"
    );
    assert_eq!(
        harness.mocks_mut().file_input().selections()[0].files,
        vec!["report.csv".to_string()]
    );
    Ok(())
}

#[test]
fn set_files_rejects_non_file_inputs() -> browser_tester::Result<()> {
    let mut harness = Harness::from_html("<input id='name'>")?;

    let error = harness
        .set_files("#name", ["report.csv"])
        .expect_err("set_files should reject non-file inputs");
    assert!(error.to_string().contains("file input control"));
    Ok(())
}

#[test]
fn document_create_element_is_publicly_supported() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'></main><div id='out'></div><script>const root = document.getElementById('root'); const created = document.createElement('span'); created.setAttribute('id', 'created'); created.textContent = 'Hello'; root.appendChild(created); document.getElementById('out').textContent = String(root.children.length) + ':' + root.children.namedItem('created').textContent;</script>",
    )?;

    harness.assert_text("#out", "1:Hello")?;
    Ok(())
}

#[test]
fn document_create_text_node_is_publicly_supported() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'></main><div id='out'></div><script>const root = document.getElementById('root'); const text = document.createTextNode('Hello'); root.appendChild(text); document.getElementById('out').textContent = root.textContent + ':' + String(text.nodeType) + ':' + text.nodeName;</script>",
    )?;

    harness.assert_text("#out", "Hello:3:#text")?;
    Ok(())
}

#[test]
fn document_create_comment_is_publicly_supported() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'></main><div id='out'></div><script>const root = document.getElementById('root'); const comment = document.createComment('Hello'); root.appendChild(comment); document.getElementById('out').textContent = root.textContent + ':' + String(comment.nodeType) + ':' + comment.nodeName + ':' + comment.textContent + ':' + String(root.childNodes.length);</script>",
    )?;

    harness.assert_text("#out", ":8:#comment::1")?;
    Ok(())
}
