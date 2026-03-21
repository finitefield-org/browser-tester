use std::collections::BTreeMap;

use bt_script::ScriptRuntime;
use bt_runtime::{MockRegistry, Session, SessionConfig};

#[test]
fn reset_all_clears_every_mock_family() {
    let mut registry = MockRegistry::default();

    registry
        .fetch_mut()
        .respond_text("https://example.test/api/message", 200, "ok");
    registry
        .fetch_mut()
        .fail("https://example.test/api/error", "network disabled");
    registry
        .fetch_mut()
        .record_call("https://example.test/api/message");

    registry.dialogs_mut().push_confirm(true);
    registry.dialogs_mut().push_prompt(Some("Ada"));
    registry.dialogs_mut().record_alert("Notice");
    registry.dialogs_mut().record_confirm("Continue?");
    registry.dialogs_mut().record_prompt("Name?");

    registry.clipboard_mut().seed_text("seeded");
    registry.clipboard_mut().record_write("copied");

    registry
        .location_mut()
        .set_current("https://example.test/next");
    registry
        .location_mut()
        .record_navigation("https://example.test/next");

    registry
        .downloads_mut()
        .capture("report.csv", b"downloaded bytes".to_vec());
    registry.open_mut().fail("popup blocked");
    registry.open_mut().record_call(
        Some("https://example.test/popup"),
        Some("_blank"),
        Some("noopener"),
    );
    registry.close_mut().fail("window closed");
    registry.close_mut().record_call();
    registry.print_mut().fail("print blocked");
    registry.print_mut().record_call();
    registry.scroll_mut().fail("scroll blocked");
    registry
        .scroll_mut()
        .record_call(bt_runtime::ScrollMethod::To, 10, 20);

    registry
        .file_input_mut()
        .set_files("#upload", ["report.csv"]);

    registry.storage_mut().seed_local("token", "abc");
    registry.storage_mut().seed_session("session-token", "xyz");

    registry.reset_all();

    assert!(registry.fetch().responses().is_empty());
    assert!(registry.fetch().errors().is_empty());
    assert!(registry.fetch().calls().is_empty());
    assert!(registry.dialogs().confirm_queue().is_empty());
    assert!(registry.dialogs().prompt_queue().is_empty());
    assert!(registry.dialogs().alert_messages().is_empty());
    assert!(registry.dialogs().confirm_messages().is_empty());
    assert!(registry.dialogs().prompt_messages().is_empty());
    assert!(registry.clipboard().seeded_text().is_none());
    assert!(registry.clipboard().writes().is_empty());
    assert!(registry.location().current_url().is_none());
    assert!(registry.location().navigations().is_empty());
    assert!(registry.open().calls().is_empty());
    assert!(registry.close().calls().is_empty());
    assert!(registry.print().calls().is_empty());
    assert!(registry.scroll().calls().is_empty());
    assert!(registry.downloads().artifacts().is_empty());
    assert!(registry.file_input().selections().is_empty());
    assert!(registry.storage().local().is_empty());
    assert!(registry.storage().session().is_empty());
}

#[test]
fn session_rejects_unseeded_window_confirm_through_script_runtime() {
    let mut session = Session::new(SessionConfig::default()).expect("session should build");
    let mut runtime = ScriptRuntime::new();

    let error = runtime
        .eval_program("window.confirm('Continue?');", "inline-script", &mut session)
        .expect_err("window.confirm should require a queued response");

    assert!(error
        .to_string()
        .contains("confirm() requires a queued response"));
}

#[test]
fn session_rejects_unseeded_window_prompt_through_script_runtime() {
    let mut session = Session::new(SessionConfig::default()).expect("session should build");
    let mut runtime = ScriptRuntime::new();

    let error = runtime
        .eval_program("window.prompt('Name?');", "inline-script", &mut session)
        .expect_err("window.prompt should require a queued response");

    assert!(error
        .to_string()
        .contains("prompt() requires a queued response"));
}

#[test]
fn session_rejects_unsupported_selector_syntax_in_closest_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root' class='primary'></main><script>document.getElementById('root').closest('main:where([data-kind=primary x])');</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("broader CSS parsing inside :where should fail explicitly");

    assert!(error.to_string().contains("Script error"));
    assert!(
        error
            .to_string()
            .contains("supported forms are #id, .class, tag, tag.class, #id.class, [attr]")
            && error.to_string().contains("optional attribute selector flags like `[attr=value i]` and `[attr=value s]`")
            && error.to_string().contains("bounded logical pseudo-classes like `:not(.primary)`")
            && error.to_string().contains("state pseudo-classes like `:checked`, `:disabled`, `:enabled`, `:indeterminate`, `:default`, `:valid`, `:invalid`, `:in-range`, and `:out-of-range`")
            && error
                .to_string()
                .contains("form-editable state pseudo-classes also include `:read-only` and `:read-write`")
            && error.to_string().contains("descendant combinators like `A B`")
            && error.to_string().contains("child combinators like `A > B`")
    );
}

#[test]
fn session_scroll_position_resets_on_navigation() {
    let mut session = Session::new(SessionConfig::default()).expect("session should build");

    session.scroll_to(10, 20).expect("scroll should succeed");
    session.scroll_by(-5, 3).expect("scroll should succeed");
    assert_eq!(session.window_scroll_x(), 5);
    assert_eq!(session.window_scroll_y(), 23);
    assert_eq!(session.window_page_x_offset(), 5);
    assert_eq!(session.window_page_y_offset(), 23);

    session
        .navigate("https://example.test/next")
        .expect("navigation should succeed");
    assert_eq!(session.window_scroll_x(), 0);
    assert_eq!(session.window_scroll_y(), 0);
}

#[test]
fn session_rejects_unsupported_has_selector_syntax_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><section id='child' class='child'></section></main><script>document.querySelector('main:has(:nth-child(2 of))');</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("malformed nth-child of selector syntax should remain unsupported");

    assert!(error.to_string().contains("Script error"));
    assert!(
        error
            .to_string()
            .contains("unsupported selector `main:has(:nth-child(2 of))`")
    );
}

#[test]
fn session_resolves_selectors_with_quoted_commas_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root' class='app'><button id='first' data-label='A,B'>First</button><button id='second' class='secondary'>Second</button></main><div id='out'></div><script>const list = document.querySelectorAll(\"button[data-label='A,B'], .secondary\"); const isMatch = document.getElementById('root').matches(\"main:is([data-label='A,B'], .app)\"); const notMatch = document.getElementById('second').matches(\"button:not([data-label='A,B'], .blocked)\"); const whereMatch = document.getElementById('root').closest(\"main:where([data-label='A,B'], .app)\"); document.getElementById('out').textContent = String(list.length) + ':' + list.item(0).textContent + ':' + list.item(1).textContent + ':' + String(isMatch) + ':' + String(notMatch) + ':' + whereMatch.textContent;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("quoted commas should remain supported inside bounded selector grammar");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "2:First:Second:true:true:FirstSecond"
    );
}

#[test]
fn session_resolves_selectors_with_escaped_punctuation_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root' class='app'><button id='foo,bar' class='alpha:beta'>First</button><button id='second' class='secondary'>Second</button><div id='out'></div></main><script>const escapedId = document.querySelector('#foo\\\\,bar'); const escapedClass = document.querySelector('.alpha\\\\:beta'); const list = document.querySelectorAll('#foo\\\\,bar, .secondary'); const isMatch = document.getElementById('root').matches('main:is(#foo\\\\)bar, .app)'); const whereMatch = document.getElementById('second').closest('button:where(#foo\\\\,bar, .secondary)'); document.getElementById('out').textContent = escapedId.textContent + ':' + escapedClass.textContent + ':' + String(list.length) + ':' + list.item(0).textContent + ':' + list.item(1).textContent + ':' + String(isMatch) + ':' + whereMatch.textContent;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("escaped punctuation should remain supported inside bounded selector grammar");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "First:First:2:First:Second:true:Second"
    );
}

#[test]
fn session_resolves_selectors_with_hex_escapes_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root' class='app'><button id='foo,bar' class='alpha:beta' data-label='foo]bar'>First</button><button id='foo)bar' class='secondary'>Second</button><div id='out'></div></main><script>const escapedId = document.querySelector('#foo\\\\2c bar'); const escapedClass = document.querySelector('.alpha\\\\3a beta'); const escapedAttr = document.querySelector('[data-label=foo\\\\5d bar]'); const whereMatch = document.getElementById('foo)bar').closest('button:where(#foo\\\\29 bar, .secondary)'); document.getElementById('out').textContent = escapedId.textContent + ':' + escapedClass.textContent + ':' + escapedAttr.textContent + ':' + whereMatch.textContent;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("hex escapes should resolve through Session-backed selector paths");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "First:First:First:Second"
    );
}

#[test]
fn session_resolves_document_title_without_title_element_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>document.title = 'Fallback'; document.getElementById('out').textContent = document.title + ':' + window.title;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("document.title should remain available without <title>");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "Fallback:Fallback"
    );
    assert_eq!(session.dom().document_title(), "Fallback");
}

#[test]
fn session_resolves_document_location_without_special_handling_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/start".to_string(),
        html: Some(
            "<main id='out'></main><script>const before = document.location; document.location = 'https://example.test/next'; const after = window.location; document.getElementById('out').textContent = before + ':' + after;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("document.location should remain available through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "https://example.test/start:https://example.test/next"
    );
    assert_eq!(session.document_location(), "https://example.test/next");
    assert_eq!(
        session.mocks().location().navigations(),
        &["https://example.test/next".to_string()]
    );
}

#[test]
fn session_resolves_document_cookie_without_special_handling_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/start".to_string(),
        html: Some(
            "<main id='out'></main><script>document.cookie = 'theme=dark'; document.cookie = 'theme=light'; document.getElementById('out').textContent = document.cookie;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("document.cookie should remain available through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "theme=light");
}

#[test]
fn session_reports_current_script_during_inline_bootstrap_and_null_elsewhere() {
    let mut session = Session::new(SessionConfig {
        url: "https://example.test/start".to_string(),
        html: Some(
            "<main id='out'></main><button id='button'></button><script id='first'>document.getElementById('out').textContent = document.currentScript.getAttribute('id');</script><script id='second'>document.getElementById('out').textContent += ':' + document.currentScript.getAttribute('id'); document.getElementById('button').addEventListener('click', () => { document.getElementById('out').textContent += ':' + String(document.currentScript); });</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("document.currentScript should remain available during inline script bootstrap");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "first:second");

    let button_id = session.dom().select("#button").unwrap()[0];
    session.click_node(button_id).unwrap();

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "first:second:null"
    );
}

#[test]
fn session_reports_document_ready_state_loading_during_bootstrap_and_complete_afterwards() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/start".to_string(),
        html: Some(
            "<main id='out'></main><script>document.getElementById('out').textContent = document.readyState;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("document.readyState should remain available during inline bootstrap");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "loading");
    assert_eq!(session.document_ready_state(), "complete");
}

#[test]
fn document_cookie_assignment_is_rejected_regression() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/start".to_string(),
        html: Some(
            "<main id='out'></main><script>document.cookie = 'badcookie';</script>".to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("document.cookie should reject malformed assignments");

    assert!(
        error
            .to_string()
            .contains("document.cookie requires `name=value`")
    );
}

#[test]
fn document_url_assignment_is_rejected_regression() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/start".to_string(),
        html: Some(
            "<main id='out'></main><script>document.URL = 'https://example.test/next';</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("document.URL should be read-only");

    assert!(error.to_string().contains("unsupported assignment target"));
    assert!(error.to_string().contains("URL"));
}

#[test]
fn document_base_uri_assignment_is_rejected_regression() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/start".to_string(),
        html: Some(
            "<main id='out'></main><script>document.baseURI = 'https://example.test/next';</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("document.baseURI should be read-only");

    assert!(error.to_string().contains("unsupported assignment target"));
    assert!(error.to_string().contains("baseURI"));
}

#[test]
fn document_origin_assignment_is_rejected_regression() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/start".to_string(),
        html: Some(
            "<main id='out'></main><script>document.origin = 'https://example.test/next';</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("document.origin should be read-only");

    assert!(error.to_string().contains("unsupported assignment target"));
    assert!(error.to_string().contains("origin"));
}

#[test]
fn document_domain_assignment_is_rejected_regression() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/start".to_string(),
        html: Some(
            "<main id='out'></main><script>document.domain = 'example.test';</script>".to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("document.domain should be read-only");

    assert!(error.to_string().contains("unsupported assignment target"));
    assert!(error.to_string().contains("domain"));
}

#[test]
fn session_resolves_any_link_pseudo_class_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><a id='docs' href='/docs'>Docs</a><a id='plain'>Plain</a><area id='map' href='/map'></main><div id='out'></div><script>const anyLink = document.querySelector(':any-link'); const links = document.querySelectorAll(':link'); const anchor = document.getElementById('docs'); const matched = anchor.matches(':link'); const closest = anchor.closest(':any-link'); document.getElementById('out').textContent = anyLink.getAttribute('id') + ':' + String(links.length) + ':' + links.item(0).getAttribute('id') + ':' + links.item(1).getAttribute('id') + ':' + String(matched) + ':' + closest.getAttribute('id');</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("any-link selector should resolve through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "docs:2:docs:map:true:docs"
    );
}

#[test]
fn session_resolves_defined_pseudo_class_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><x-widget id='widget'></x-widget><svg id='svg'><text id='svg-text'>Hi</text></svg></main><div id='out'></div><script>const defined = document.querySelectorAll(':defined'); const widget = document.getElementById('widget'); const svg = document.getElementById('svg'); document.getElementById('out').textContent = defined.item(0).getAttribute('id') + ':' + defined.item(1).getAttribute('id') + ':' + defined.item(2).getAttribute('id') + ':' + String(widget.matches(':defined')) + ':' + String(svg.matches(':defined'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("defined selector should resolve through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "root:svg:svg-text:false:true"
    );
}

#[test]
fn session_resolves_dir_pseudo_class_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root' dir='rtl'><section id='section'><div id='child'>Child</div></section><p id='ltr' dir='ltr'>LTR</p><div id='auto' dir='auto'><span id='auto-child'>Auto</span></div></main><div id='out'></div><script>const dir = document.querySelector(':dir(rtl)'); const dirAll = document.querySelectorAll(':dir(rtl)'); const section = document.querySelector('#section:dir(rtl)'); const child = document.getElementById('child').closest(':dir(rtl)'); const autoChild = document.querySelector('#auto-child:dir(rtl)'); document.getElementById('out').textContent = dir.getAttribute('id') + ':' + String(dirAll.length) + ':' + section.getAttribute('id') + ':' + child.getAttribute('id') + ':' + autoChild.getAttribute('id');</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("dir selector should resolve through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "root:5:section:child:auto-child"
    );
}

#[test]
fn session_resolves_placeholder_shown_pseudo_class_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><input id='name' placeholder='Name'><input id='filled' placeholder='Filled' value='Ada'><textarea id='bio' placeholder='Bio'></textarea></main><div id='out'></div><script>const before = document.querySelectorAll(':placeholder-shown'); document.getElementById('name').value = 'Alice'; document.getElementById('bio').value = 'Bio text'; const after = document.querySelectorAll(':placeholder-shown'); const name = document.getElementById('name'); document.getElementById('out').textContent = String(before.length) + ':' + before.item(0).getAttribute('id') + ':' + String(after.length) + ':' + String(name.matches(':placeholder-shown')) + ':' + String(document.getElementById('filled').matches(':placeholder-shown'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("placeholder-shown selector should resolve through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "2:name:0:false:false"
    );
}

#[test]
fn session_resolves_indeterminate_pseudo_class_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><progress id='loading'></progress><form id='signup'><input type='radio' name='mode' id='mode-a'><input type='radio' name='mode' id='mode-b'></form><form id='chosen'><input type='radio' name='picked' id='picked-a' checked><input type='radio' name='picked' id='picked-b'></form><div id='out'></div><script>const before = document.querySelectorAll(':indeterminate'); document.getElementById('mode-b').setAttribute('checked', ''); const after = document.querySelectorAll(':indeterminate'); document.getElementById('out').textContent = String(before.length) + ':' + before.item(0).getAttribute('id') + ':' + String(after.length) + ':' + String(document.getElementById('picked-a').matches(':indeterminate')) + ':' + String(document.getElementById('loading').matches(':indeterminate'));</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("indeterminate selector should resolve through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "3:loading:1:false:true"
    );
}

#[test]
fn session_resolves_read_only_and_read_write_pseudo_classes_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><input id='name' value='Ada'><input id='readonly' value='Bee' readonly><textarea id='bio'>Hello</textarea><div id='editable' contenteditable='true'>Edit</div><select id='mode'><option value='a'>A</option></select><button id='button'>Button</button><div id='out'></div><script>const readWrite = document.querySelectorAll(':read-write'); const readOnly = document.querySelectorAll(':read-only'); document.getElementById('out').textContent = String(readWrite.length) + ':' + readWrite.item(0).getAttribute('id') + ':' + readWrite.item(1).getAttribute('id') + ':' + readWrite.item(2).getAttribute('id') + ':' + String(readOnly.item(0).matches(':read-only')) + ':' + String(document.getElementById('readonly').matches(':read-only')) + ':' + String(document.getElementById('mode').matches(':read-only')) + ':' + String(document.getElementById('button').matches(':read-only'));</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("read-only/read-write selectors should resolve through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "3:name:bio:editable:true:true:true:true"
    );
}

#[test]
fn session_resolves_valid_and_invalid_pseudo_classes_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><input id='filled' type='text' required value='Ada'><input id='empty' type='text' required><input id='check' type='checkbox' required><input id='check-ok' type='checkbox' required checked><textarea id='bio' required></textarea><select id='mode' required><option value='a' selected>A</option><option value='b'>B</option></select><div id='out'></div><script>document.getElementById('empty').value = 'Bee'; document.getElementById('check').setAttribute('checked', '');</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("valid/invalid selectors should resolve through Session");

    let valid_ids = session.dom().select(":valid").unwrap();
    let invalid_ids = session.dom().select(":invalid").unwrap();
    let filled_id = session.dom().select("#filled").unwrap()[0];
    let empty_id = session.dom().select("#empty").unwrap()[0];
    let check_id = session.dom().select("#check").unwrap()[0];
    let check_ok_id = session.dom().select("#check-ok").unwrap()[0];
    let bio_id = session.dom().select("#bio").unwrap()[0];
    let mode_id = session.dom().select("#mode").unwrap()[0];

    assert_eq!(
        valid_ids,
        vec![filled_id, empty_id, check_id, check_ok_id, mode_id]
    );
    assert_eq!(invalid_ids, vec![bio_id]);
}

#[test]
fn session_resolves_in_range_and_out_of_range_pseudo_classes_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><input id='low' type='number' min='2' max='6' value='1'><input id='high' type='number' min='2' max='6' value='7'><input id='in-range' type='number' min='2' max='6' value='4'><div id='out'></div><script>const inRange = document.querySelectorAll(':in-range'); const outOfRange = document.querySelectorAll(':out-of-range'); document.getElementById('out').textContent = String(inRange.length) + ':' + inRange.item(0).getAttribute('id') + ':' + String(outOfRange.length) + ':' + outOfRange.item(0).getAttribute('id') + ':' + outOfRange.item(1).getAttribute('id');</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("in-range/out-of-range selectors should resolve through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "1:in-range:2:low:high"
    );
}

#[test]
fn session_rejects_read_only_selector_syntax_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><input id='name' value='Ada'></main><div id='out'></div><script>document.querySelector(':read-only()');</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("malformed read-only selector should fail explicitly");

    assert!(error.to_string().contains("Script error"));
    assert!(
        error
            .to_string()
            .contains("unsupported selector `:read-only()`")
    );
}

#[test]
fn session_reflects_attributes_through_inline_script_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><button id='button'>First</button><input id='name'><input id='agree' type='checkbox'><select id='mode'><option value='a'>A</option><option id='selected' value='b'>B</option></select><div id='out'></div><script>const button = document.getElementById('button'); button.setAttribute('class', 'primary'); button.toggleAttribute('data-flag'); const name = document.getElementById('name'); name.setAttribute('value', 'Alice'); const agree = document.getElementById('agree'); agree.setAttribute('checked', ''); document.getElementById('selected').setAttribute('selected', ''); document.getElementById('out').textContent = String(document.querySelectorAll('.primary').length) + ':' + String(document.querySelectorAll('[data-flag]').length) + ':' + String(button.getAttribute('data-label')) + ':' + name.value + ':' + String(agree.checked) + ':' + document.querySelector('option:checked').value;</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("attribute reflection should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "1:1:null:Alice:true:b"
    );
    assert_eq!(session.dom().select(".primary").unwrap().len(), 1);
    assert_eq!(session.dom().select("[data-flag]").unwrap().len(), 1);
    assert_eq!(session.dom().select("input:checked").unwrap().len(), 1);
    assert_eq!(session.dom().select("option:checked").unwrap().len(), 1);
}

#[test]
fn session_reflects_class_views_through_inline_script_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><button id='button' class='base' data-kind='App'>First</button><div id='out'></div><script>const button = document.getElementById('button'); button.className = 'primary secondary'; const before = button.classList.length; const contains = button.classList.contains('primary'); button.classList.add('tertiary'); button.classList.remove('secondary'); const toggled = button.classList.toggle('active'); button.dataset.userId = '42'; document.getElementById('out').textContent = button.className + ':' + String(before) + ':' + String(contains) + ':' + String(toggled) + ':' + button.dataset.kind + ':' + button.dataset.userId + ':' + String(button.classList) + ':' + String(button.dataset);</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("class views should remain wired through Session");

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
fn session_rejects_out_of_range_hex_escape_selectors_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'></main><script>document.querySelector('#foo\\\\110000 bar');</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("out-of-range hex escapes should fail explicitly");

    assert!(error.to_string().contains("Script error"));
    assert!(
        error
            .to_string()
            .contains("supported forms are #id, .class, tag, tag.class, #id.class, [attr]")
            && error.to_string().contains("optional attribute selector flags like `[attr=value i]` and `[attr=value s]`")
            && error.to_string().contains("bounded logical pseudo-classes like `:not(.primary)`")
            && error.to_string().contains("state pseudo-classes like `:checked`, `:disabled`, `:enabled`, `:indeterminate`, `:default`, `:valid`, `:invalid`, `:in-range`, and `:out-of-range`")
            && error
                .to_string()
                .contains("form-editable state pseudo-classes also include `:read-only` and `:read-write`")
            && error.to_string().contains("descendant combinators like `A B`")
            && error.to_string().contains("child combinators like `A > B`")
    );
}

#[test]
fn session_rejects_control_character_hex_escape_selectors_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='foo'></main><script>document.querySelector('#foo\\\\0 bar');</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("control-character hex escapes should fail explicitly");

    assert!(error.to_string().contains("Script error"));
    assert!(
        error
            .to_string()
            .contains("unsupported selector `#foo\\0 bar`")
    );
}

#[test]
fn session_resolves_html_collection_named_item_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><span name='alpha'>First</span><span id='second'>Second</span></main><div id='out'></div><script>const children = document.getElementById('root').children; const alpha = children.namedItem('alpha'); document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = alpha.textContent + ':' + String(children.namedItem('alpha'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("namedItem should remain available");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "First:null");
}

#[test]
fn session_resolves_radio_node_list_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><form id='signup'><input type='radio' name='mode' id='mode-a' value='a'><input type='radio' name='mode' id='mode-b' value='b'></form></div><div id='out'></div><script>const elements = document.getElementById('signup').elements; const named = elements.namedItem('mode'); const before = named.length; document.getElementById('signup').innerHTML += '<input type=\"radio\" name=\"mode\" id=\"mode-c\" value=\"c\" checked>'; document.getElementById('out').textContent = String(before) + ':' + String(named.length) + ':' + named.item(0).value + ':' + named.item(1).value + ':' + named.value + ':' + String(named);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("radio node list should remain available");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "2:3:a:b:c:[object RadioNodeList]"
    );
}

#[test]
fn session_resolves_radio_node_list_entries_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><form id='signup'><input type='radio' name='mode' id='mode-a' value='a'><input type='radio' name='mode' id='mode-b' value='b'></form></div><div id='out'></div><script>const elements = document.getElementById('signup').elements; const named = elements.namedItem('mode'); const entries = named.entries(); const first = entries.next(); const second = entries.next(); const third = entries.next(); document.getElementById('out').textContent = String(named.length) + ':' + String(first.value.index) + ':' + first.value.value + ':' + String(second.value.index) + ':' + second.value.value + ':' + String(third.done);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("radio node list entries should remain available");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "2:0:[object Element]:1:[object Element]:true"
    );
}

#[test]
fn session_clears_radio_node_list_value_when_no_radio_matches_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><form id='signup'><input type='radio' name='mode' id='mode-a' value='a' checked><input type='radio' name='mode' id='mode-b' value='b'></form></div><div id='out'></div><script>const named = document.getElementById('signup').elements.namedItem('mode'); named.value = 'missing'; document.getElementById('out').textContent = named.value + ':' + String(document.getElementById('mode-a').checked) + ':' + String(document.getElementById('mode-b').checked) + ':' + String(named.item(0).checked) + ':' + String(named.item(1).checked);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("radio node list value assignment should clear unmatched groups");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        ":false:false:false:false"
    );
}

#[test]
fn session_resolves_document_scripts_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><script id='first-script'></script></div><div id='out'></div><script>const out = document.getElementById('out'); const scripts = document.scripts; const before = scripts.length; const first = scripts.namedItem('first-script'); document.getElementById('root').textContent = 'gone'; out.textContent = String(before) + ':' + String(scripts.length) + ':' + String(first) + ':' + String(scripts.namedItem('missing'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("document.scripts should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "2:1:[object Element]:null"
    );
}

#[test]
fn session_resolves_document_style_sheets_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><style id='first-style'>.primary { color: red; }</style><link id='first-link' rel='stylesheet' href='a.css'><link id='ignored-link' rel='preload' href='b.css'></div><div id='out'></div><script>const out = document.getElementById('out'); const sheets = document.styleSheets; const before = sheets.length; document.getElementById('first-link').setAttribute('rel', 'preload'); out.textContent = String(before) + ':' + String(sheets.length) + ':' + String(sheets.item(0)) + ':' + String(sheets.item(1));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("document.styleSheets should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "2:1:[object CSSStyleSheet]:null"
    );
}

#[test]
fn session_resolves_document_style_sheets_entries_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><style id='first-style'>.primary { color: red; }</style><link id='first-link' rel='stylesheet' href='a.css'><link id='ignored-link' rel='preload' href='b.css'></div><div id='out'></div><script>const out = document.getElementById('out'); const sheets = document.styleSheets; const keys = sheets.keys(); const values = sheets.values(); const entries = sheets.entries(); const key = keys.next(); const value = values.next(); const entry = entries.next(); out.textContent = String(sheets.length) + ':' + String(key.value) + ':' + String(value.value) + ':' + String(entry.value.index) + ':' + String(entry.value.value);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("document.styleSheets iterator helpers should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "2:0:[object CSSStyleSheet]:0:[object CSSStyleSheet]"
    );
}

#[test]
fn session_resolves_document_style_sheets_named_item_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><style id='first-style'>.primary { color: red; }</style><link id='first-link' rel='stylesheet' href='a.css'><link id='ignored-link' rel='preload' href='b.css'></div><div id='out'></div><script>const out = document.getElementById('out'); const sheets = document.styleSheets; const before = sheets.length; const first = sheets.namedItem('first-style'); const second = sheets.namedItem('first-link'); document.getElementById('root').textContent = 'gone'; out.textContent = String(before) + ':' + String(sheets.length) + ':' + String(first) + ':' + String(second) + ':' + String(sheets.namedItem('missing'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("document.styleSheets namedItem should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "2:0:[object CSSStyleSheet]:[object CSSStyleSheet]:null"
    );
}

#[test]
fn session_resolves_document_applets_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><applet id='first-applet' name='first-applet'>First</applet><applet name='second-applet'>Second</applet></div><div id='out'></div><script>const applets = document.applets; const before = applets.length; const first = applets.namedItem('first-applet'); document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(before) + ':' + String(applets.length) + ':' + String(first) + ':' + String(applets.namedItem('missing'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("document.applets should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "2:0:[object Element]:null"
    );
}

#[test]
fn session_resolves_document_anchors_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><a name='first'>First</a><a id='ignored'>Ignored</a></div><div id='out'></div><script>const anchors = document.anchors; const before = anchors.length; const first = anchors.namedItem('first'); document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(before) + ':' + String(anchors.length) + ':' + String(first) + ':' + String(anchors.namedItem('missing'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("document.anchors should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "1:0:[object Element]:null"
    );
}

#[test]
fn session_resolves_document_children_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><span>First</span></main><div id='out'></div><script>const children = document.children; const before = children.length; const first = children.item(0); const root = children.namedItem('root'); document.getElementById('root').remove(); document.getElementById('out').textContent = String(before) + ':' + String(children.length) + ':' + String(first) + ':' + String(root) + ':' + String(children.namedItem('root'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("document.children should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "3:2:[object Element]:[object Element]:null"
    );
    assert_eq!(session.dom().select("#root").unwrap().len(), 0);
}

#[test]
fn session_resolves_child_nodes_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'>Hello<span>World</span></main><div id='out'></div><script>const rootNode = document.childNodes.item(0); const nodes = rootNode.childNodes; const before = nodes.length; const first = nodes.item(0); document.getElementById('root').innerHTML += '<!--tail-->'; document.getElementById('out').textContent = String(before) + ':' + String(nodes.length) + ':' + first.nodeName + ':' + String(nodes.item(1).nodeType) + ':' + nodes.item(2).nodeName;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("childNodes should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "2:3:#text:1:#comment"
    );
}

#[test]
fn session_resolves_template_content_live_collection_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<template id='tpl'><span id='inner'>Inner</span></template><div id='out'></div><script>const tpl = document.getElementById('tpl'); const content = tpl.content; const nodes = content.childNodes; const children = content.children; const before = nodes.length; tpl.innerHTML += '<!--tail--><span id=\"second\">Second</span>'; document.getElementById('out').textContent = String(content) + ':' + String(before) + ':' + String(nodes.length) + ':' + nodes.item(1).nodeName + ':' + String(children.length) + ':' + String(children.namedItem('second').textContent);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("template content collections should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "[object DocumentFragment]:1:3:#comment:2:Second"
    );
}

#[test]
fn session_resolves_template_content_inner_html_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<template id='tpl'><span id='inner'>Inner</span></template><div id='out'></div><script>const tpl = document.getElementById('tpl'); const content = tpl.content; const before = content.innerHTML; content.innerHTML = '<!--tail--><span id=\"second\">Second</span>'; document.getElementById('out').textContent = before + '|' + content.innerHTML + '|' + String(content.childNodes.length) + ':' + content.childNodes.item(0).nodeName + ':' + String(content.children.length) + ':' + content.children.namedItem('second').textContent;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("template content innerHTML should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "<span id=\"inner\">Inner</span>|<!--tail--><span id=\"second\">Second</span>|2:#comment:1:Second"
    );
    assert_eq!(session.dom().select("#second").unwrap().len(), 1);
}

#[test]
fn session_serializes_namespace_aware_names_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some("<main id='root'><svg id='icon' viewbox='0 0 10 10'><foreignobject id='foreign'><div id='html'>Text</div></foreignobject></svg><math id='formula' definitionurl='https://example.com'><mi id='symbol'>x</mi></math><div id='out'></div><script>const icon = document.getElementById('icon'); const formula = document.getElementById('formula'); document.getElementById('out').textContent = icon.outerHTML + '|' + formula.outerHTML;</script></main>".to_string()),
        local_storage: BTreeMap::new(),
    })
    .expect("namespace-aware serialization should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "<svg id=\"icon\" viewBox=\"0 0 10 10\"><foreignObject id=\"foreign\"><div id=\"html\">Text</div></foreignObject></svg>|<math definitionURL=\"https://example.com\" id=\"formula\"><mi id=\"symbol\">x</mi></math>"
    );
}

#[test]
fn session_rejects_table_rows_on_non_table_elements_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='bad'></div><script>document.getElementById('bad').rows.length;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-table rows access should fail explicitly");

    let message = error.to_string();
    assert!(message.contains("Script error"));
    assert!(message.contains("table.rows"));
    assert!(message.contains("supported table.rows host element"));
}

#[test]
fn session_rejects_row_cells_on_non_row_elements_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='bad'></div><script>document.getElementById('bad').cells.length;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-row cells access should fail explicitly");

    let message = error.to_string();
    assert!(message.contains("Script error"));
    assert!(message.contains("tr.cells"));
    assert!(message.contains("supported tr.cells host element"));
}

#[test]
fn session_rejects_document_applets_on_elements_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='wrapper'><div id='not-doc'></div></div><script>document.getElementById('not-doc').applets.length;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-document applets access should fail explicitly");

    let message = error.to_string();
    assert!(message.contains("`applets`"));
}

#[test]
fn session_resolves_window_children_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><span id='first'>First</span><span id='second'>Second</span></div><div id='out'></div><script>const children = document.defaultView.children; document.getElementById('out').textContent = String(children.length) + ':' + children.item(0).textContent + ':' + children.item(1).textContent + ':' + String(children.namedItem('first')) + ':' + String(children.namedItem('missing'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("window.children should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "3:FirstSecond::null:null"
    );
}

#[test]
fn session_rejects_window_frames_length_assignment_regression() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<iframe id='first'></iframe><script>window.frames.length = 2;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("window.frames.length should be read-only");

    let message = error.to_string();
    assert!(message.contains("cannot assign to `length` on html collection value"));
}

#[test]
fn session_rejects_window_frame_element_assignment_regression() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some("<script>window.frameElement = 2;</script>".to_string()),
        local_storage: BTreeMap::new(),
    })
    .expect_err("window.frameElement should be read-only");

    let message = error.to_string();
    assert!(message.contains("unsupported assignment target"));
}

#[test]
fn session_rejects_window_opener_assignment_regression() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some("<script>window.opener = 2;</script>".to_string()),
        local_storage: BTreeMap::new(),
    })
    .expect_err("window.opener should be read-only");

    let message = error.to_string();
    assert!(message.contains("unsupported assignment target"));
}

#[test]
fn session_rejects_form_length_assignment_regression() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<form id='signup'><input><input></form><script>document.getElementById('signup').length = 2;</script>".to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("form.length should be read-only");

    let message = error.to_string();
    assert!(message.contains("unsupported assignment target"));
}

#[test]
fn session_rejects_window_length_assignment_regression() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some("<iframe id='first'></iframe><script>window.length = 2;</script>".to_string()),
        local_storage: BTreeMap::new(),
    })
    .expect_err("window.length should be read-only");

    let message = error.to_string();
    assert!(message.contains("unsupported assignment target"));
}

#[test]
fn session_exposes_document_compat_mode_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>document.getElementById('out').textContent = document.compatMode;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("document.compatMode should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "CSS1Compat");
}

#[test]
fn session_exposes_document_character_set_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>document.getElementById('out').textContent = document.characterSet + ':' + document.charset;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("document.characterSet should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "UTF-8:UTF-8");
}

#[test]
fn session_exposes_document_content_type_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>document.getElementById('out').textContent = document.contentType;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("document.contentType should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "text/html");
}

#[test]
fn session_exposes_document_visibility_state_and_hidden_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>document.getElementById('out').textContent = document.visibilityState + ':' + String(document.hidden);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("document.visibilityState should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "visible:false");
}

#[test]
fn session_exposes_window_device_pixel_ratio_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>document.getElementById('out').textContent = String(window.devicePixelRatio);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("window.devicePixelRatio should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "1");
    assert_eq!(session.window_device_pixel_ratio(), 1.0);
}

#[test]
fn session_exposes_window_inner_width_and_inner_height_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>document.getElementById('out').textContent = String(window.innerWidth) + ':' + String(window.innerHeight);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("window.innerWidth and window.innerHeight should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "1024:768");
    assert_eq!(session.window_inner_width(), 1024);
    assert_eq!(session.window_inner_height(), 768);
}

#[test]
fn session_exposes_window_outer_width_and_outer_height_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>document.getElementById('out').textContent = String(window.outerWidth) + ':' + String(window.outerHeight);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("window.outerWidth and window.outerHeight should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "1024:768");
    assert_eq!(session.window_outer_width(), 1024);
    assert_eq!(session.window_outer_height(), 768);
}

#[test]
fn session_exposes_window_screen_position_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>document.getElementById('out').textContent = String(window.screenX) + ':' + String(window.screenY) + ':' + String(window.screenLeft) + ':' + String(window.screenTop);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("window.screenX / screenY / screenLeft / screenTop should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "0:0:0:0");
    assert_eq!(session.window_screen_x(), 0);
    assert_eq!(session.window_screen_y(), 0);
    assert_eq!(session.window_screen_left(), 0);
    assert_eq!(session.window_screen_top(), 0);
}

#[test]
fn session_exposes_window_screen_object_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>document.getElementById('out').textContent = String(window.screen) + ':' + String(window.screen.width) + ':' + String(window.screen.height) + ':' + String(window.screen.availWidth) + ':' + String(window.screen.availHeight) + ':' + String(window.screen.availLeft) + ':' + String(window.screen.availTop) + ':' + String(window.screen.colorDepth) + ':' + String(window.screen.pixelDepth);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("window.screen should remain wired through Session");

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
fn session_exposes_window_screen_orientation_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>document.getElementById('out').textContent = String(window.screen.orientation) + ':' + window.screen.orientation.type + ':' + String(window.screen.orientation.angle);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("window.screen.orientation should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "[object ScreenOrientation]:landscape-primary:0"
    );
}

#[test]
fn session_exposes_document_referrer_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>document.getElementById('out').textContent = '[' + document.referrer + ']';</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("document.referrer should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "[]");
}

#[test]
fn session_rejects_window_screen_orientation_assignment_regression() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<script>window.screen.orientation.type = 'portrait-primary';</script>".to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("window.screen.orientation.type should be rejected explicitly");

    assert!(error.to_string().contains("screen orientation"));
    assert!(error.to_string().contains("type"));
}

#[test]
fn session_exposes_window_name_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>const before = window.name; window.self.name = 'updated'; document.getElementById('out').textContent = before + ':' + window.window.name + ':' + window.parent.name + ':' + window.top.name;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("window.name should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        ":updated:updated:updated"
    );
}

#[test]
fn session_rejects_window_self_assignment_regression() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some("<script>window.self = 'updated';</script>".to_string()),
        local_storage: BTreeMap::new(),
    })
    .expect_err("window.self should be rejected explicitly");

    assert!(error.to_string().contains("unsupported assignment target"));
    assert!(error.to_string().contains("self"));
}

#[test]
fn session_exposes_window_closed_accessor_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>document.getElementById('out').textContent = String(window.closed) + ':' + String(window.self.closed) + ':' + String(window.window.closed) + ':' + String(window.parent.closed) + ':' + String(window.top.closed);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("window.closed should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "false:false:false:false:false"
    );
}

#[test]
fn session_rejects_window_closed_assignment_regression() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some("<script>window.closed = true;</script>".to_string()),
        local_storage: BTreeMap::new(),
    })
    .expect_err("window.closed should be rejected explicitly");

    assert!(error.to_string().contains("unsupported assignment target"));
    assert!(error.to_string().contains("closed"));
}

#[test]
fn session_resolves_window_history_accessor_regression() {
    let mut session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>document.getElementById('out').textContent = String(window.history) + ':' + String(window.history.length) + ':' + String(window.history.state) + ':' + String(window.history.scrollRestoration);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("window.history should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "[object History]:1:null:auto"
    );
    assert_eq!(session.window_history_length(), 1);

    session.navigate("https://example.test/next").unwrap();
    assert_eq!(session.window_history_length(), 2);
}

#[test]
fn session_updates_window_history_state_via_push_and_replace_state_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>window.history.pushState('step-1', '', 'https://example.test/step-1'); window.history.replaceState('step-2', '', 'https://example.test/step-2'); document.getElementById('out').textContent = document.location + ':' + String(window.history.length) + ':' + String(window.history.state);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("window.history.pushState and replaceState should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "https://example.test/step-2:2:step-2"
    );
    assert_eq!(session.window_history_length(), 2);
    assert_eq!(session.window_history_state(), Some("step-2"));
}

#[test]
fn session_rejects_window_history_assignment_regression() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some("<script>window.history.length = 2;</script>".to_string()),
        local_storage: BTreeMap::new(),
    })
    .expect_err("window.history should be rejected explicitly");

    assert!(error.to_string().contains("history"));
    assert!(error.to_string().contains("length"));
}

#[test]
fn session_rejects_window_history_state_assignment_regression() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some("<script>window.history.state = 'step';</script>".to_string()),
        local_storage: BTreeMap::new(),
    })
    .expect_err("window.history.state should be rejected explicitly");

    assert!(error.to_string().contains("history"));
    assert!(error.to_string().contains("state"));
}

#[test]
fn session_rejects_window_history_push_state_with_too_few_arguments_regression() {
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
fn session_rejects_window_history_replace_state_with_too_few_arguments_regression() {
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
fn session_updates_window_history_scroll_restoration_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>window.history.scrollRestoration = 'manual'; document.getElementById('out').textContent = String(window.history.scrollRestoration);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("window.history.scrollRestoration should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "manual");
}

#[test]
fn session_rejects_window_history_scroll_restoration_assignment_regression() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some("<script>window.history.scrollRestoration = 'sideways';</script>".to_string()),
        local_storage: BTreeMap::new(),
    })
    .expect_err("window.history.scrollRestoration should be rejected explicitly");

    assert!(error.to_string().contains("scroll restoration"));
}

#[test]
fn session_exposes_document_dir_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root' dir='ltr'><div id='out'></div><script>const before = document.dir; document.dir = 'rtl'; document.getElementById('out').textContent = before + ':' + document.dir + ':' + document.documentElement.getAttribute('dir');</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("document.dir should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "ltr:rtl:rtl");
}

#[test]
fn session_exposes_document_root_head_and_body_with_null_fallbacks() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><div id='out'></div><script>const root = document.documentElement; const scrolling = document.scrollingElement; document.getElementById('out').textContent = root.getAttribute('id') + ':' + String(document.head) + ':' + String(document.body) + ':' + scrolling.getAttribute('id');</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("fragment documents should expose a root element and null head/body fallbacks");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "root:null:null:root"
    );
}

#[test]
fn session_resolves_document_embeds_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><embed id='first-embed'><embed name='second-embed'></div><div id='out'></div><script>const embeds = document.embeds; const before = embeds.length; const first = embeds.namedItem('first-embed'); document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(before) + ':' + String(embeds.length) + ':' + String(first) + ':' + String(embeds.namedItem('missing'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("document.embeds should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "2:0:[object Element]:null"
    );
}

#[test]
fn session_resolves_document_plugins_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><embed id='first-embed'><embed name='second-embed'></div><div id='out'></div><script>const plugins = document.plugins; const before = plugins.length; const first = plugins.namedItem('first-embed'); document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(before) + ':' + String(plugins.length) + ':' + String(first) + ':' + String(plugins.namedItem('missing'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("document.plugins should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "2:0:[object Element]:null"
    );
}

#[test]
fn session_rejects_labels_on_non_labelable_elements_explicitly() {
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
fn session_resolves_node_list_for_each_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><span class='item'>First</span><span class='item'>Second</span></main><div id='out'></div><script>const nodes = document.querySelectorAll('.item'); nodes.forEach((item, index, list) => { document.getElementById('out').textContent += String(index) + ':' + item.textContent + ':' + String(list.length) + ';'; }, null);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("NodeList.forEach should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "0:First:2;1:Second:2;"
    );
}

#[test]
fn session_resolves_collection_iterator_helpers_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><span class='item'>First</span><span class='item'>Second</span></main><div id='out'></div><script>const nodes = document.querySelectorAll('.item'); const nodeValues = nodes.values(); const nodeKeys = nodes.keys(); const children = document.getElementById('root').children; const childValues = children.values(); const childKeys = children.keys(); document.getElementById('root').textContent = 'gone'; const firstNode = nodeValues.next(); const secondNode = nodeValues.next(); const thirdNode = nodeValues.next(); const firstKey = nodeKeys.next(); const secondKey = nodeKeys.next(); const thirdKey = nodeKeys.next(); const firstChild = childValues.next(); const secondChild = childValues.next(); const thirdChild = childValues.next(); const childFirstKey = childKeys.next(); const childSecondKey = childKeys.next(); const childThirdKey = childKeys.next(); document.getElementById('out').textContent = firstNode.value.textContent + ':' + String(firstNode.done) + ':' + secondNode.value.textContent + ':' + String(secondNode.done) + ':' + String(thirdNode.done) + ':' + String(firstKey.value) + ':' + String(secondKey.value) + ':' + String(thirdKey.done) + ':' + firstChild.value.textContent + ':' + String(firstChild.done) + ':' + secondChild.value.textContent + ':' + String(secondChild.done) + ':' + String(thirdChild.done) + ':' + String(childFirstKey.value) + ':' + String(childSecondKey.value) + ':' + String(childThirdKey.done);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("collection iterator helpers should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "First:false:Second:false:true:0:1:true:First:false:Second:false:true:0:1:true"
    );
}

#[test]
fn session_resolves_collection_entries_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><span class='item'>One</span><span class='item'>Two</span></main><div id='out'></div><script>const docEntries = document.childNodes.entries(); const childEntries = document.getElementById('root').children.entries(); const firstDoc = docEntries.next(); const secondDoc = docEntries.next(); const firstChild = childEntries.next(); const secondChild = childEntries.next(); document.getElementById('out').textContent = String(firstDoc.value.index) + ':' + firstDoc.value.value.nodeName + ':' + String(secondDoc.value.index) + ':' + secondDoc.value.value.nodeName + ':' + String(firstChild.value.index) + ':' + firstChild.value.value.textContent + ':' + String(secondChild.value.index) + ':' + secondChild.value.value.textContent;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("collection entries should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "0:main:1:div:0:One:1:Two"
    );
}

#[test]
fn session_resolves_get_elements_by_tag_name_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><span name='alpha'>First</span><span id='second'>Second</span></main><div id='out'></div><script>const all = document.getElementsByTagName('span'); const scoped = document.getElementById('root').getElementsByTagName('span'); const alpha = all.namedItem('alpha'); const before = all.length; document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(before) + ':' + String(all.length) + ':' + String(scoped.length) + ':' + alpha.textContent + ':' + String(all.namedItem('alpha'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("getElementsByTagName should remain available");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "2:0:0:First:null"
    );
}

#[test]
fn session_resolves_get_elements_by_class_name_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root' class='alpha'><span name='alpha' class='alpha'>First</span><span id='second' class='alpha'>Second</span></main><div id='out'></div><script>const all = document.getElementsByClassName('alpha'); const scoped = document.getElementById('root').getElementsByClassName('alpha'); const named = all.namedItem('alpha'); const root = all.item(0); const before = all.length; const beforeScoped = scoped.length; document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(before) + ':' + String(all.length) + ':' + String(beforeScoped) + ':' + String(scoped.length) + ':' + named.textContent + ':' + String(scoped.namedItem('alpha')) + ':' + root.textContent;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("getElementsByClassName should remain available");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "3:1:2:0:First:null:gone"
    );
}

#[test]
fn session_resolves_get_elements_by_tag_name_ns_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><svg id='icon'><rect id='rect'></rect><circle id='dot'></circle></svg><math id='formula'><mi id='symbol'>x</mi></math><span id='label'>Label</span></div><div id='out'></div><script>const svgAll = document.getElementsByTagNameNS('http://www.w3.org/2000/svg', '*'); const svgRect = document.getElementById('icon').getElementsByTagNameNS('http://www.w3.org/2000/svg', 'rect'); const dot = svgAll.namedItem('dot'); document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(svgAll.length) + ':' + String(svgRect.length) + ':' + String(dot) + ':' + String(svgAll.namedItem('dot'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("getElementsByTagNameNS should remain available");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "0:1:[object Element]:null"
    );
}

#[test]
fn session_resolves_nth_child_with_non_element_siblings_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main>lead<!-- gap --><button id='first'>First</button><button id='second'>Second</button><div id='out'></div><script>const second = document.querySelector('button:nth-child(2)'); const first = document.getElementById('first'); document.getElementById('out').textContent = second.textContent + ':' + String(first.matches('button:nth-child(1)')) + ':' + String(second.matches('button:nth-child(2)'));</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("nth-child should ignore non-element siblings");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "Second:true:true"
    );
}

#[test]
fn session_resolves_nth_child_formulas_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main>lead<!-- gap --><button id='first'>First</button><button id='second'>Second</button><button id='third'>Third</button><div id='out'></div><script>const odd = document.querySelectorAll('button:nth-child(odd)'); const even = document.querySelector('button:nth-child(even)'); const formula = document.querySelectorAll('button:nth-child(2n+1)'); const limited = document.querySelectorAll('button:nth-child(-n+2)'); document.getElementById('out').textContent = String(odd.length) + ':' + even.textContent + ':' + String(formula.length) + ':' + String(limited.length);</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("nth-child formulas should remain available");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "2:Second:2:2");
}

#[test]
fn session_resolves_nth_last_child_formulas_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main>lead<!-- gap --><button id='first'>First</button><button id='second'>Second</button><button id='third'>Third</button></main><div id='out'></div><script>const second = document.querySelector('button:nth-last-child(2)'); const odd = document.querySelectorAll('button:nth-last-child(odd)'); const even = document.querySelectorAll('button:nth-last-child(even)'); const formula = document.querySelector('button:nth-last-child(2n+1)'); document.getElementById('out').textContent = second.textContent + ':' + String(odd.length) + ':' + String(even.length) + ':' + formula.textContent;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("nth-last-child formulas should remain available");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "Second:2:1:First"
    );
}

#[test]
fn session_resolves_not_pseudo_class_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root' class='app' data-kind='APP READY'><button id='first' class='primary'>First</button><button id='disabled' class='primary' disabled>Disabled</button><button id='enabled' class='secondary'>Enabled</button><div id='out'></div><script>const enabled = document.querySelectorAll('button:not(:disabled)'); const second = document.getElementById('enabled'); const root = second.closest('main:not([data-kind~=blocked i], .blocked)'); const bounded = document.querySelectorAll('button:not(main > .secondary, :disabled)'); document.getElementById('out').textContent = String(enabled.length) + ':' + enabled.item(0).textContent + ':' + enabled.item(1).textContent + ':' + String(second.matches('button:not(.primary)')) + ':' + String(root.matches('main:not([data-kind~=blocked i], .blocked)')) + ':' + document.querySelector('button:not(:nth-child(even))').textContent + ':' + String(bounded.length) + ':' + bounded.item(0).textContent;</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect(":not pseudo-class should remain available");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "2:First:Enabled:true:true:First:1:First"
    );
}

#[test]
fn session_resolves_is_pseudo_class_and_nested_selector_lists_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root' class='app' data-kind='APP READY' lang='EN-US'><button id='first' class='primary'>First</button><button id='disabled' class='primary' disabled>Disabled</button><button id='enabled' class='secondary'>Enabled</button></main><div id='out'></div><script>const outer = document.querySelectorAll('main, button:is(.primary, .secondary)'); const filtered = document.querySelectorAll('button:is(:disabled, .secondary)'); const bounded = document.querySelectorAll('button:is(main > .secondary, :disabled)'); const second = document.getElementById('enabled'); const root = second.closest('main:is([lang|=en i], .blocked)'); document.getElementById('out').textContent = String(outer.length) + ':' + String(filtered.length) + ':' + String(bounded.length) + ':' + String(second.matches('button:is(.secondary, .blocked)')) + ':' + String(root.matches('main:is([lang|=en i], .blocked)'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect(":is pseudo-class should remain available");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "4:2:2:true:true"
    );
}

#[test]
fn session_resolves_where_pseudo_class_and_nested_selector_lists_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root' class='app' data-kind='APP READY' lang='EN-US'><button id='first' class='primary'>First</button><button id='disabled' class='primary' disabled>Disabled</button><button id='enabled' class='secondary'>Enabled</button></main><div id='out'></div><script>const outer = document.querySelectorAll('main, button:where(.primary, .secondary)'); const filtered = document.querySelectorAll('button:where(:disabled, .secondary)'); const bounded = document.querySelectorAll('button:where(main > .secondary, :disabled)'); const second = document.getElementById('enabled'); const root = second.closest('main:where([lang|=en i], .blocked)'); document.getElementById('out').textContent = String(outer.length) + ':' + String(filtered.length) + ':' + String(bounded.length) + ':' + String(second.matches('button:where(.secondary, .blocked)')) + ':' + String(root.matches('main:where([lang|=en i], .blocked)'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect(":where pseudo-class should remain available");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "4:2:2:true:true"
    );
}

#[test]
fn session_resolves_root_and_empty_pseudo_classes_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><div id='empty'></div><div id='comment-only'><!-- marker --></div><div id='with-text'>x</div><div id='out'></div></main><script>const root = document.querySelector(':root'); const empty = document.querySelector('#empty:empty'); const commentOnly = document.querySelector('#comment-only:empty'); const nonEmpty = document.querySelector('#with-text:empty'); const isRoot = document.getElementById('root').matches(':root'); const childIsRoot = document.getElementById('empty').matches(':root'); const closestRoot = document.getElementById('with-text').closest(':root'); document.getElementById('out').textContent = String(root.matches(':root')) + ':' + String(empty.matches(':empty')) + ':' + String(commentOnly.matches(':empty')) + ':' + String(nonEmpty) + ':' + String(isRoot) + ':' + String(childIsRoot) + ':' + String(closestRoot.matches(':root'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect(":root and :empty pseudo-classes should remain available");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "true:true:true:null:true:false:true"
    );
}

#[test]
fn session_resolves_only_child_and_only_of_type_pseudo_classes_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'>lead<!-- gap --><div id='single-child-parent'>text<!-- marker --><section id='only-child'>child</section><!-- marker --></div><div id='type-parent'><span id='first-span'>one</span><em id='only-of-type'>type</em><span id='second-span'>two</span></div><div id='out'></div><script>const onlyChild = document.querySelector('#only-child:only-child'); const onlyOfType = document.querySelector('#only-of-type:only-of-type'); const onlyChildMatches = document.querySelectorAll('#single-child-parent > :only-child'); const onlyOfTypeMatches = document.querySelectorAll('#type-parent > :only-of-type'); const firstSpan = document.getElementById('first-span'); const parent = onlyChild.closest('#single-child-parent'); document.getElementById('out').textContent = onlyChild.textContent + ':' + onlyOfType.textContent + ':' + String(onlyChildMatches.length) + ':' + String(onlyOfTypeMatches.length) + ':' + String(firstSpan.matches('#first-span:not(:only-child)')) + ':' + String(firstSpan.matches('#first-span:not(:only-of-type)')) + ':' + String(parent.matches('#single-child-parent'));</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect(":only-child and :only-of-type pseudo-classes should remain available");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "child:type:1:1:true:true:true"
    );
}

#[test]
fn session_resolves_first_last_and_nth_of_type_pseudo_classes_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><div id='type-parent'><span id='first-span' class='skip'>one</span><em id='first-em'>first</em><span id='middle-span' class='match'>two</span><em id='last-em'>last</em><span id='last-span' class='match'>three</span></div><div id='out'></div><script>const firstSpan = document.querySelector('#first-span:first-of-type'); const lastSpan = document.querySelector('#last-span:last-of-type'); const middleSpan = document.querySelector('#middle-span:nth-of-type(2)'); const filteredMiddle = document.querySelector('#middle-span:nth-of-type(1 of .match)'); const filteredLast = document.querySelector('#last-span:nth-last-of-type(1 of .match)'); const middleFromEnd = document.querySelector('#middle-span:nth-last-of-type(2)'); const firstEm = document.querySelector('#first-em:first-of-type'); const lastEm = document.querySelector('#last-em:last-of-type'); document.getElementById('out').textContent = String(firstSpan.matches('#first-span:first-of-type')) + ':' + String(lastSpan.matches('#last-span:last-of-type')) + ':' + String(middleSpan.matches('#middle-span:nth-of-type(2)')) + ':' + String(filteredMiddle.matches('#middle-span:nth-of-type(1 of .match)')) + ':' + String(filteredLast.matches('#last-span:nth-last-of-type(1 of .match)')) + ':' + String(middleFromEnd.matches('#middle-span:nth-last-of-type(2)')) + ':' + String(firstEm.matches('#first-em:first-of-type')) + ':' + String(lastEm.matches('#last-em:last-of-type'));</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect(":first-of-type and :nth-of-type pseudo-classes should remain available");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "true:true:true:true:true:true:true:true"
    );
}

#[test]
fn session_rejects_unsupported_nth_of_type_selector_syntax_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><section id='child'>child</section></main><script>document.querySelector('#child:nth-of-type(1 of .child, )');</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("malformed :nth-of-type selector should fail explicitly");

    assert!(error.to_string().contains("Script error"));
    assert!(error.to_string().contains("unsupported selector"));
    assert!(error.to_string().contains(".child,"));
}

#[test]
fn session_rejects_unsupported_empty_pseudo_arguments_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'></main><script>document.querySelector('main:empty(1)');</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("unsupported :empty arguments should fail explicitly");

    assert!(error.to_string().contains("Script error"));
    assert!(
        error
            .to_string()
            .contains("unsupported selector `main:empty(1)`")
    );
}

#[test]
fn session_rejects_unsupported_first_of_type_selector_syntax_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><section id='child'>child</section></main><script>document.querySelector('#child:first-of-type()');</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("malformed :first-of-type selector should fail explicitly");

    assert!(error.to_string().contains("Script error"));
    assert!(
        error
            .to_string()
            .contains("unsupported selector `#child:first-of-type()`")
    );
}

#[test]
fn session_rejects_unsupported_only_child_selector_syntax_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><section id='child'>child</section></main><script>document.querySelector('#child:only-child()');</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("malformed :only-child selector should fail explicitly");

    assert!(error.to_string().contains("Script error"));
    assert!(
        error
            .to_string()
            .contains("unsupported selector `#child:only-child()`")
    );
}

#[test]
fn session_rejects_unsupported_not_argument_syntax_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root' class='primary'></main><script>document.getElementById('root').matches('main:not([data-kind=primary x])');</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("broader CSS parsing inside :not should fail explicitly");

    assert!(error.to_string().contains("Script error"));
    assert!(
        error
            .to_string()
            .contains("supported forms are #id, .class, tag, tag.class, #id.class, [attr]")
            && error.to_string().contains("optional attribute selector flags like `[attr=value i]` and `[attr=value s]`")
            && error.to_string().contains("bounded logical pseudo-classes like `:not(.primary)`")
            && error.to_string().contains("state pseudo-classes like `:checked`, `:disabled`, `:enabled`, `:indeterminate`, `:default`, `:valid`, `:invalid`, `:in-range`, and `:out-of-range`")
            && error
                .to_string()
                .contains("form-editable state pseudo-classes also include `:read-only` and `:read-write`")
            && error.to_string().contains("descendant combinators like `A B`")
            && error.to_string().contains("child combinators like `A > B`")
    );
}

#[test]
fn session_rejects_unsupported_lang_arguments_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root' lang='en'><script>document.querySelector(':lang()');</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("malformed :lang selector should fail explicitly");

    assert!(error.to_string().contains("Script error"));
    assert!(error.to_string().contains("unsupported selector `:lang()`"));
}

#[test]
fn session_resolves_lang_selector_with_language_ranges() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root' lang='en-US'><section id='section'><div id='child'>Child</div></section><p id='french' lang='fr'>French</p><div id='out'></div><script>const root = document.querySelector(':lang(en, fr)'); const closest = document.getElementById('child').closest(':lang(fr, en)'); document.getElementById('out').textContent = root.getAttribute('id') + ':' + closest.getAttribute('id');</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("language ranges should be supported inside :lang()");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "root:child");
}

#[test]
fn session_resolves_focus_pseudo_classes() {
    let mut session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><section id='section'><input id='field'></section><div id='outside'>outside</div></main><div id='out'></div><script>document.getElementById('field').addEventListener('focus', () => { const field = document.querySelector(':focus'); const section = document.getElementById('section'); const root = document.getElementById('root'); document.getElementById('out').textContent = field.getAttribute('id') + ':' + String(section.matches(':focus-within')) + ':' + String(root.matches(':focus-within')); });</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("focus pseudo-classes should resolve through Session");

    let field_id = session.dom().select("#field").unwrap()[0];
    let section_id = session.dom().select("#section").unwrap()[0];
    let root_id = session.dom().select("#root").unwrap()[0];
    session.focus_node(field_id).expect("focus should work");

    assert_eq!(session.dom().select(":focus").unwrap(), vec![field_id]);
    assert_eq!(
        session.dom().select("#field:focus").unwrap(),
        vec![field_id]
    );
    assert_eq!(
        session.dom().select("#section:focus-within").unwrap(),
        vec![section_id]
    );
    assert_eq!(
        session.dom().select("#root:focus-within").unwrap(),
        vec![root_id]
    );
    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "field:true:true"
    );

    session.blur_node(field_id).expect("blur should work");
    assert!(session.dom().select(":focus").unwrap().is_empty());
    assert!(
        session
            .dom()
            .select("#section:focus-within")
            .unwrap()
            .is_empty()
    );
}

#[test]
fn session_resolves_document_active_element_regression() {
    let mut session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<input id='field'><div id='out'></div><script>document.getElementById('field').addEventListener('focus', () => { document.getElementById('out').textContent = document.activeElement.getAttribute('id'); });</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("document.activeElement should remain wired through Session");

    let field_id = session.dom().select("#field").unwrap()[0];
    let out_id = session.dom().select("#out").unwrap()[0];
    session.focus_node(field_id).expect("focus should work");

    assert_eq!(session.dom().text_content_for_node(out_id), "field");
}

#[test]
fn session_resolves_document_has_focus_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='out'></div><script>document.getElementById('out').textContent = String(document.hasFocus());</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("document.hasFocus should be wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "false");
}

#[test]
fn session_resolves_window_navigator_on_line_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='out'></div><script>document.getElementById('out').textContent = String(window.navigator.onLine) + ':' + String(window.navigator.webdriver) + ':' + String(window.navigator.appCodeName) + ':' + String(window.navigator.appName) + ':' + String(window.navigator.appVersion) + ':' + String(window.navigator.product) + ':' + String(window.navigator.productSub) + ':' + String(window.navigator.vendor) + ':' + String(window.navigator.vendorSub) + ':' + String(window.navigator.pdfViewerEnabled) + ':' + String(window.navigator.doNotTrack) + ':' + String(window.navigator.javaEnabled()) + ':' + String(window.navigator.hardwareConcurrency) + ':' + String(window.navigator.maxTouchPoints);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("navigator.onLine should be wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "true:false:browser-tester-next:browser-tester-next:browser-tester-next:browser-tester-next:browser-tester-next:browser-tester-next:browser-tester-next:false:unspecified:false:8:0"
    );
}

#[test]
fn session_resolves_window_navigator_plugins_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><embed id='first-embed'><embed name='second-embed'></div><div id='out'></div><script>document.getElementById('out').textContent = String(window.navigator.plugins.length);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("navigator.plugins should be wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "2");
}

#[test]
fn session_resolves_target_pseudo_classes() {
    let mut session = Session::new(SessionConfig {
        url: "https://example.test/app#target".to_string(),
        html: Some(
            "<main id='root'><section id='target'>Target</section><a id='fallback' name='fallback'>Fallback</a><span name='named'>Named</span></main><div id='out'></div><script>const target = document.querySelector(':target'); document.getElementById('out').textContent = target.textContent + ':' + String(target.matches(':target'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("target pseudo-classes should resolve through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "Target:true");
    assert_eq!(
        session.dom().select(":target").unwrap()[0],
        session.dom().select("#target").unwrap()[0]
    );

    session
        .navigate("https://example.test/app#fallback")
        .expect("navigation should update target fragment");
    assert_eq!(
        session.dom().select(":target").unwrap()[0],
        session.dom().select("#fallback").unwrap()[0]
    );

    session
        .navigate("https://example.test/app#named")
        .expect("navigation should update target fragment");
    assert_eq!(
        session.dom().select(":target").unwrap()[0],
        session.dom().select("[name=named]").unwrap()[0]
    );
}

#[test]
fn session_rejects_document_links_on_elements_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='wrapper'><div id='not-doc'></div></div><script>document.getElementById('not-doc').links.length;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-document links access should fail explicitly");

    assert!(error.to_string().contains("Script error"));
    assert!(error.to_string().contains("unsupported member access"));
    assert!(error.to_string().contains("`links`"));
    assert!(error.to_string().contains("element value"));
}

#[test]
fn session_rejects_document_all_on_elements_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='wrapper'><div id='not-doc'></div></div><script>document.getElementById('not-doc').all.length;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-document all access should fail explicitly");

    assert!(error.to_string().contains("Script error"));
    assert!(error.to_string().contains("unsupported member access"));
    assert!(error.to_string().contains("`all`"));
    assert!(error.to_string().contains("element value"));
}

#[test]
fn session_rejects_get_elements_by_name_on_elements_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><span name='alpha'>First</span></main><script>document.getElementById('root').getElementsByName('alpha');</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("element.getElementsByName should fail explicitly");

    assert!(error.to_string().contains("Script error"));
    assert!(
        error
            .to_string()
            .contains("unsupported Element method: getElementsByName")
    );
}

#[test]
fn session_rejects_form_elements_on_non_form_elements_explicitly() {
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
fn session_rejects_select_options_on_non_select_elements_explicitly() {
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
fn session_rejects_select_options_add_on_datalist_elements_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><select id='mode'><option id='first' value='a'>A</option></select><datalist id='list'><option id='extra' value='b'>B</option></datalist><script>document.getElementById('list').options.add(document.getElementById('extra'));</script></div>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("datalist options should not support add()");

    assert!(error.to_string().contains("Script error"));
    assert!(error.to_string().contains("node is not a select element"));
}

#[test]
fn session_rejects_select_selected_options_on_non_select_elements_explicitly() {
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
fn session_rejects_map_areas_on_non_map_elements_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='wrapper'><div id='not-map'></div></div><script>document.getElementById('not-map').areas.length;</script>"
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
fn session_rejects_table_t_bodies_on_non_table_elements_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='wrapper'><div id='not-table'></div></div><script>document.getElementById('not-table').tBodies.length;</script>"
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
fn session_reorders_nodes_with_before_and_after_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><section id='source'><button id='second'>Second</button><button id='third'>Third</button></section><button id='first'>First</button><div id='out'></div><script>const source = document.getElementById('source'); const first = document.getElementById('first'); const second = document.getElementById('second'); const third = document.getElementById('third'); second.before(first); second.after(third); document.getElementById('out').textContent = String(source.children.length) + ':' + source.children.item(0).textContent + ':' + source.children.item(1).textContent + ':' + source.children.item(2).textContent + ':' + String(document.querySelectorAll('#source > button').length) + ':' + document.querySelector('#first').textContent + ':' + document.querySelector('#third').textContent;</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("before/after tree mutation should remain available");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "3:First:Second:Third:3:First:Third"
    );
}

#[test]
fn session_rejects_tree_mutation_cycles_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><section id='child'><span id='grandchild'>x</span></section></main><script>document.getElementById('child').appendChild(document.getElementById('root'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("ancestor insertion should fail explicitly");

    assert!(error.to_string().contains("cannot insert"));
}

#[test]
fn session_serializes_inner_html_and_outer_html_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><section id='target'><button id='old' class='primary'>Old</button></section><div id='out'></div><script>const target = document.getElementById('target'); const before = target.innerHTML; target.innerHTML = '<span id=\"first\">One</span><span id=\"second\">Two</span>'; const after = target.innerHTML; const replacement = document.getElementById('root').querySelector('#target'); replacement.outerHTML = '<article id=\"replacement\"><em id=\"inner\">Inner</em></article>'; document.getElementById('out').textContent = before + '|' + after + '|' + String(document.querySelector('#target')) + ':' + document.getElementById('replacement').outerHTML + ':' + document.getElementById('inner').textContent;</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("HTML serialization surfaces should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "<button class=\"primary\" id=\"old\">Old</button>|<span id=\"first\">One</span><span id=\"second\">Two</span>|null:<article id=\"replacement\"><em id=\"inner\">Inner</em></article>:Inner"
    );
    assert!(session.dom().select("#old").unwrap().is_empty());
    assert_eq!(session.dom().select("#replacement").unwrap().len(), 1);
    assert_eq!(session.dom().select("#inner").unwrap().len(), 1);
}

#[test]
fn session_serializes_insert_adjacent_html_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><section id='target'><button id='old' class='primary'>Old</button></section></main><div id='out'></div><script>const target = document.getElementById('target'); target.insertAdjacentHTML('beforebegin', '<aside id=\"before\">Before</aside>'); target.insertAdjacentHTML('afterbegin', '<span id=\"first\">First</span>'); target.insertAdjacentHTML('beforeend', '<span id=\"last\">Last</span>'); target.insertAdjacentHTML('afterend', '<aside id=\"after\">After</aside>'); document.getElementById('out').textContent = document.getElementById('root').innerHTML + '|' + target.innerHTML + '|' + String(target.children.length) + ':' + String(document.querySelector('#before')) + ':' + String(document.querySelector('#after'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("insertAdjacentHTML should remain wired through Session");

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
fn session_rejects_detached_insert_adjacent_html_regression() {
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
fn session_mutation_hardening_updates_live_collections_and_selectors_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><form id='form'><input id='first' name='first' value='one'></form><select id='mode'><option value='a'>A</option></select><div id='out'></div><script>const form = document.getElementById('form'); const select = document.getElementById('mode'); const formsBefore = document.forms.length; const inputsBefore = document.querySelectorAll('input').length; form.outerHTML = '<div id=\"form-replacement\"></div>'; select.innerHTML = '<option id=\"second\" value=\"b\" selected>B</option><option id=\"third\" value=\"c\">C</option>'; document.getElementById('out').textContent = formsBefore + ':' + document.forms.length + ':' + inputsBefore + ':' + document.querySelectorAll('input').length + ':' + select.options.length + ':' + document.querySelector('option:checked').value;</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("mutation hardening should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "1:0:1:0:2:b");
    assert_eq!(session.dom().select("#form-replacement").unwrap().len(), 1);
    assert_eq!(session.dom().select("#third").unwrap().len(), 1);
    assert!(session.dom().select("#form").unwrap().is_empty());
    assert_eq!(session.dom().select("option:checked").unwrap().len(), 1);
}

#[test]
fn session_rejects_lossy_outer_html_serialization_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><div id='target'></div><div id='out'></div><script>const target = document.getElementById('target'); target.setAttribute('data-label', \"a'b\\\"c\"); document.getElementById('out').textContent = String(target.outerHTML);</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("lossy serialization should fail explicitly");

    assert!(error.to_string().contains("Script error"));
    assert!(error.to_string().contains("contains both quote types"));
}

#[test]
fn session_rejects_malformed_html_fragment_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><section id='target'></section><script>document.getElementById('target').innerHTML = '<span></main>';</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("malformed innerHTML fragments should fail explicitly");

    assert!(error.to_string().contains("Script error"));
    assert!(error.to_string().contains("mismatched closing tag"));
}

#[test]
fn session_rejects_storage_method_wrong_arity_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>window.localStorage.setItem('token');</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("storage methods should validate arity explicitly");

    assert!(error.to_string().contains("Script error"));
    assert!(
        error
            .to_string()
            .contains("setItem() expects exactly two arguments")
    );
}

#[test]
fn session_rejects_unseeded_match_media_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>window.matchMedia('(prefers-color-scheme: dark)').matches;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("matchMedia should require a configured seed");

    assert!(error.to_string().contains("Script error"));
    assert!(
        error
            .to_string()
            .contains("no matchMedia mock configured for `(prefers-color-scheme: dark)`")
    );
}
