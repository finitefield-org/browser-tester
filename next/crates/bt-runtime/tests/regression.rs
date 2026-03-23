use std::collections::BTreeMap;

use bt_runtime::{MatchMediaListenerCall, MockRegistry, Session, SessionConfig};
use bt_script::ScriptRuntime;

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
        .eval_program(
            "window.confirm('Continue?');",
            "inline-script",
            &mut session,
        )
        .expect_err("window.confirm should require a queued response");

    assert!(
        error
            .to_string()
            .contains("confirm() requires a queued response")
    );
}

#[test]
fn session_rejects_unseeded_window_prompt_through_script_runtime() {
    let mut session = Session::new(SessionConfig::default()).expect("session should build");
    let mut runtime = ScriptRuntime::new();

    let error = runtime
        .eval_program("window.prompt('Name?');", "inline-script", &mut session)
        .expect_err("window.prompt should require a queued response");

    assert!(
        error
            .to_string()
            .contains("prompt() requires a queued response")
    );
}

#[test]
fn session_rejects_window_navigator_mime_types_assignment_through_script_runtime() {
    let mut session = Session::new(SessionConfig::default()).expect("session should build");
    let mut runtime = ScriptRuntime::new();

    let error = runtime
        .eval_program(
            "window.navigator.mimeTypes = null;",
            "inline-script",
            &mut session,
        )
        .expect_err("window.navigator.mimeTypes should be read-only");

    assert!(
        error
            .to_string()
            .contains("cannot assign to `mimeTypes` on navigator value")
    );
}

#[test]
fn session_resolves_window_navigator_mime_types_iterator_helpers_through_script_runtime() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some("<div id='out'></div>".to_string()),
        local_storage: BTreeMap::new(),
    })
    .expect("session should build");
    let mut runtime = ScriptRuntime::new();
    let mut session = session;

    runtime
        .eval_program(
            "document.getElementById('out').textContent = String(window.navigator.mimeTypes.keys().next().done) + ':' + String(window.navigator.mimeTypes.values().next().done) + ':' + String(window.navigator.mimeTypes.entries().next().done);",
            "inline-script",
            &mut session,
        )
        .expect("navigator.mimeTypes iterator helpers should resolve through Session");

    assert_eq!(
        session
            .dom()
            .text_content_for_node(session.dom().select("#out").unwrap()[0]),
        "true:true:true"
    );
}

#[test]
fn session_resolves_window_navigator_mime_types_named_item_through_script_runtime() {
    let mut session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some("<div id='out'></div>".to_string()),
        local_storage: BTreeMap::new(),
    })
    .expect("session should build");
    let mut runtime = ScriptRuntime::new();

    runtime
        .eval_program(
            "document.getElementById('out').textContent = String(window.navigator.mimeTypes.namedItem('missing'));",
            "inline-script",
            &mut session,
        )
        .expect("window.navigator.mimeTypes.namedItem should resolve through Session");

    assert_eq!(
        session
            .dom()
            .text_content_for_node(session.dom().select("#out").unwrap()[0]),
        "null"
    );
}

#[test]
fn session_resolves_window_navigator_languages_iterator_helpers_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='out'></div><script>const languages = window.navigator.languages; const keys = languages.keys(); const values = languages.values(); const entries = languages.entries(); const firstKey = keys.next(); const firstValue = values.next(); const firstEntry = entries.next(); const secondKey = keys.next(); const secondValue = values.next(); const secondEntry = entries.next(); document.getElementById('out').textContent = String(firstKey.value) + ':' + String(firstValue.value) + ':' + String(firstEntry.value.index) + ':' + firstEntry.value.value + ':' + String(secondKey.done) + ':' + String(secondValue.done) + ':' + String(secondEntry.done);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("navigator.languages iterator helpers should be wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "0:en-US:0:en-US:true:true:true"
    );
}

#[test]
fn session_resolves_window_navigator_plugins_iterator_helpers_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><embed id='first-embed'><embed id='second-embed'></div><div id='out'></div><script>const plugins = window.navigator.plugins; const keys = plugins.keys(); const values = plugins.values(); const entries = plugins.entries(); const firstKey = keys.next(); const firstValue = values.next(); const firstEntry = entries.next(); let out = ''; plugins.forEach((element, index, list) => { out += String(index) + ':' + element.getAttribute('id') + ':' + String(list.length) + ';'; }); document.getElementById('out').textContent = String(firstKey.value) + ':' + firstValue.value.getAttribute('id') + ':' + String(firstEntry.value.index) + ':' + firstEntry.value.value.getAttribute('id') + ':' + out;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("navigator.plugins iterator helpers should be wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "0:first-embed:0:first-embed:0:first-embed:2;1:second-embed:2;"
    );
}

#[test]
fn session_rejects_window_navigator_plugins_refresh_extra_arguments_regression() {
    let mut session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some("<div id='out'></div>".to_string()),
        local_storage: BTreeMap::new(),
    })
    .expect("navigator.plugins.refresh should be wired through Session");
    let mut runtime = ScriptRuntime::new();

    let error = runtime
        .eval_program(
            "document.plugins.refresh(1);",
            "inline-script",
            &mut session,
        )
        .expect_err("navigator.plugins.refresh should reject extra arguments");

    assert!(
        error
            .to_string()
            .contains("navigator.plugins.refresh() expects no arguments")
    );
}

#[test]
fn session_rejects_unsupported_selector_syntax_in_closest_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root' class='primary'></main><script>document.getElementById('root').closest('main:where([data-kind=primary x y])');</script>"
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
fn session_resolves_location_assign_replace_and_reload_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>const before = document.location; window.location.assign('https://example.test/assign'); document.location.replace('https://example.test/replace'); window.location.reload(); document.getElementById('out').textContent = before + ':' + document.location + ':' + String(window.history.length);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("location methods should remain available through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "https://example.test/app:https://example.test/replace:2"
    );
    assert_eq!(session.document_location(), "https://example.test/replace");
    assert_eq!(
        session.mocks().location().navigations(),
        &[
            "https://example.test/assign".to_string(),
            "https://example.test/replace".to_string(),
            "https://example.test/replace".to_string(),
        ]
    );
}

#[test]
fn session_resolves_location_href_getter_and_setter_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>const before = window.location.href; document.location.href = 'https://example.test/next'; const after = document.location.href; document.getElementById('out').textContent = before + ':' + after;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("location.href should remain available through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "https://example.test/app:https://example.test/next"
    );
    assert_eq!(session.document_location(), "https://example.test/next");
    assert_eq!(
        session.mocks().location().navigations(),
        &["https://example.test/next".to_string()]
    );
}

#[test]
fn session_supports_match_media_listeners_regression() {
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
    .expect("matchMedia listeners should remain wired through Session");

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
fn session_resolves_location_stringification_without_special_handling_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test:8443/start?x#old".to_string(),
        html: Some(
            "<main id='out'></main><script>document.getElementById('out').textContent = document.location.toString() + ':' + window.location.valueOf();</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("location stringification should remain available through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "https://example.test:8443/start?x#old:https://example.test:8443/start?x#old"
    );
}

#[test]
fn session_resolves_location_hash_getter_and_setter_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>const before = window.location.hash; document.location.hash = '#next'; const after = document.location.hash; document.getElementById('out').textContent = before + ':' + document.location + ':' + after;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("location.hash should remain available through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        ":https://example.test/app#next:#next"
    );
    assert_eq!(session.document_location(), "https://example.test/app#next");
    assert_eq!(
        session.mocks().location().navigations(),
        &["https://example.test/app#next".to_string()]
    );
}

#[test]
fn session_resolves_location_pathname_getter_and_setter_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/start?x#old".to_string(),
        html: Some(
            "<main id='out'></main><script>const before = window.location.pathname; document.location.pathname = 'next'; const after = document.location.pathname; document.getElementById('out').textContent = before + ':' + document.location + ':' + after;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("location.pathname should remain available through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "/start:https://example.test/next?x#old:/next"
    );
    assert_eq!(
        session.document_location(),
        "https://example.test/next?x#old"
    );
    assert_eq!(
        session.mocks().location().navigations(),
        &["https://example.test/next?x#old".to_string()]
    );
}

#[test]
fn session_resolves_location_search_getter_and_setter_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/start?x#old".to_string(),
        html: Some(
            "<main id='out'></main><script>const before = window.location.search; document.location.search = '?next'; const after = document.location.search; document.getElementById('out').textContent = before + ':' + document.location + ':' + after;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("location.search should remain available through Session");

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
fn session_resolves_location_origin_getter_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test:8443/start?x#old".to_string(),
        html: Some(
            "<main id='out'></main><script>const before = window.location.origin; document.location.pathname = 'next'; const after = document.location.origin; document.getElementById('out').textContent = before + ':' + after + ':' + document.location;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("location.origin should remain available through Session");

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
}

#[test]
fn session_resolves_location_protocol_host_hostname_and_port_getters_and_setters_regression() {
    let session = Session::new(SessionConfig {
        url: "https://app.local:8443/start?x#old".to_string(),
        html: Some(
            "<main id='out'></main><script>const before = window.location.protocol + '|' + window.location.host + '|' + window.location.hostname + '|' + window.location.port; document.location.protocol = 'http:'; document.location.host = 'example.test:8080'; document.location.hostname = 'example.test'; document.location.port = '8080'; const after = window.location.protocol + '|' + window.location.host + '|' + window.location.hostname + '|' + window.location.port; document.getElementById('out').textContent = before + ':' + after + ':' + document.location;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("location protocol/host/hostname/port should remain available through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "https:|app.local:8443|app.local|8443:http:|example.test:8080|example.test|8080:http://example.test:8080/start?x#old"
    );
    assert_eq!(
        session.document_location(),
        "http://example.test:8080/start?x#old"
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
}

#[test]
fn session_resolves_location_username_and_password_getters_and_setters_regression() {
    let session = Session::new(SessionConfig {
        url: "https://alice:secret@example.test:8443/start?x#old".to_string(),
        html: Some(
            "<main id='out'></main><script>const before = window.location.username + '|' + window.location.password; document.location.username = 'bob'; document.location.password = 'hunter2'; document.location.port = '9444'; const after = window.location.username + '|' + window.location.password + '|' + window.location.port; document.getElementById('out').textContent = before + ':' + after + ':' + document.location;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("location username/password should remain available through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "alice|secret:bob|hunter2|9444:https://bob:hunter2@example.test:9444/start?x#old"
    );
    assert_eq!(
        session.document_location(),
        "https://bob:hunter2@example.test:9444/start?x#old"
    );
    assert_eq!(
        session.mocks().location().navigations(),
        &[
            "https://bob:secret@example.test:8443/start?x#old".to_string(),
            "https://bob:hunter2@example.test:8443/start?x#old".to_string(),
            "https://bob:hunter2@example.test:9444/start?x#old".to_string(),
        ]
    );
}

#[test]
fn session_rejects_location_port_with_non_numeric_value_regression() {
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
fn session_rejects_location_assign_without_url_regression() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some("<script>window.location.assign();</script>".to_string()),
        local_storage: BTreeMap::new(),
    })
    .expect_err("location.assign should reject missing URLs");

    assert!(
        error
            .to_string()
            .contains("location.assign() expects exactly one argument")
    );
}

#[test]
fn session_rejects_location_href_without_url_regression() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some("<script>window.location.href = '';</script>".to_string()),
        local_storage: BTreeMap::new(),
    })
    .expect_err("location.href should reject missing URLs");

    assert!(
        error
            .to_string()
            .contains("navigate() requires a non-empty URL")
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
fn document_first_child_assignment_is_rejected_regression() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/start".to_string(),
        html: Some(
            "<main id='out'></main><script>document.firstChild = null;</script>".to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("document.firstChild should be read-only");

    assert!(error.to_string().contains("unsupported assignment target"));
    assert!(error.to_string().contains("firstChild"));
}

#[test]
fn node_next_sibling_assignment_is_rejected_regression() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/start".to_string(),
        html: Some(
            "<main id='root'><span id='child'>Child</span><span id='sibling'>Sibling</span></main><script>document.getElementById('child').nextSibling = null;</script>".to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("nextSibling should be read-only");

    assert!(error.to_string().contains("unsupported assignment target"));
    assert!(error.to_string().contains("on element"));
    assert!(error.to_string().contains("nextSibling"));
}

#[test]
fn element_next_element_sibling_assignment_is_rejected_regression() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/start".to_string(),
        html: Some(
            "<main id='root'><span id='child'>Child</span><span id='sibling'>Sibling</span></main><script>document.getElementById('child').nextElementSibling = null;</script>".to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("nextElementSibling should be read-only");

    assert!(error.to_string().contains("unsupported assignment target"));
    assert!(error.to_string().contains("on element"));
    assert!(error.to_string().contains("nextElementSibling"));
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
fn session_resolves_checkbox_indeterminate_state_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><input id='toggle' type='checkbox'><div id='out'></div><script>const toggle = document.getElementById('toggle'); toggle.indeterminate = true; const matches = document.querySelectorAll(':indeterminate'); document.getElementById('out').textContent = String(matches.length) + ':' + matches.item(0).getAttribute('id') + ':' + String(toggle.matches(':indeterminate'));</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("checkbox indeterminate state should resolve through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "1:toggle:true");
}

#[test]
fn session_resolves_checkbox_default_checked_state_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><input id='toggle' type='checkbox'><div id='out'></div><script>const toggle = document.getElementById('toggle'); const before = toggle.defaultChecked; toggle.defaultChecked = true; const matches = document.querySelectorAll(':default'); document.getElementById('out').textContent = String(before) + ':' + String(toggle.defaultChecked) + ':' + String(toggle.checked) + ':' + String(matches.length) + ':' + matches.item(0).getAttribute('id') + ':' + String(toggle.matches(':default'));</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("checkbox defaultChecked state should resolve through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "false:true:true:1:toggle:true"
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
fn session_reflects_option_selected_through_inline_script_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><select id='mode'><option id='first' value='a'>A</option><option id='second' value='b'>B</option></select><div id='out'></div><script>const second = document.getElementById('second'); const before = second.selected; second.selected = true; const selected = document.getElementById('mode').selectedOptions; document.getElementById('out').textContent = String(before) + ':' + String(second.selected) + ':' + String(selected.length) + ':' + selected.item(0).getAttribute('id') + ':' + document.querySelector('option:checked').getAttribute('id');</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("option.selected reflection should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "false:true:1:second:second"
    );
    assert_eq!(session.dom().select("option:checked").unwrap().len(), 1);
    assert_eq!(session.dom().select("#second[selected]").unwrap().len(), 1);
}

#[test]
fn session_reflects_option_default_selected_through_inline_script_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><select id='mode'><option id='first' value='a'>A</option><option id='second' value='b'>B</option></select><div id='out'></div><script>const second = document.getElementById('second'); const before = second.defaultSelected; second.defaultSelected = true; const selected = document.getElementById('mode').selectedOptions; document.getElementById('out').textContent = String(before) + ':' + String(second.defaultSelected) + ':' + String(selected.length) + ':' + selected.item(0).getAttribute('id') + ':' + document.querySelector('option:checked').getAttribute('id');</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("option.defaultSelected reflection should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "false:true:1:second:second"
    );
    assert_eq!(session.dom().select("option:checked").unwrap().len(), 1);
    assert_eq!(session.dom().select("#second[selected]").unwrap().len(), 1);
}

#[test]
fn session_reflects_select_selected_index_through_inline_script_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><select id='mode'><option id='first' value='a' selected>A</option><option id='second' value='b'>B</option><option id='third' value='c'>C</option></select><div id='out'></div><script>const select = document.getElementById('mode'); const before = select.selectedIndex; select.selectedIndex = 2; document.getElementById('out').textContent = String(before) + ':' + String(select.selectedIndex) + ':' + String(document.querySelector('option:checked').getAttribute('id'));</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("select.selectedIndex reflection should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "0:2:third");
    assert_eq!(session.dom().select("option:checked").unwrap().len(), 1);
    assert_eq!(session.dom().select("#third[selected]").unwrap().len(), 1);
}

#[test]
fn session_reflects_select_type_through_inline_script_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><select id='mode'><option id='first' value='a'>A</option></select><div id='out'></div><script>const select = document.getElementById('mode'); const before = select.type; select.multiple = true; const after = select.type; document.getElementById('out').textContent = before + ':' + after;</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("select.type reflection should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "select-one:select-multiple"
    );
}

#[test]
fn session_reflects_option_index_through_inline_script_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><select id='mode'><option id='first' value='a'>A</option><option id='second' value='b'>B</option></select><div id='out'></div><script>const select = document.getElementById('mode'); const second = document.getElementById('second'); const before = second.index; select.insertAdjacentHTML('afterbegin', '<option id=\"zero\" value=\"z\">Z</option>'); document.getElementById('out').textContent = String(before) + ':' + String(second.index) + ':' + String(document.getElementById('zero').index);</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("option.index reflection should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "1:2:0");
    assert_eq!(session.dom().select("#zero").unwrap().len(), 1);
}

#[test]
fn session_reflects_option_form_through_inline_script_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><form id='owner'><select id='mode'><option id='first' value='a'>A</option><option id='second' value='b'>B</option></select></form><div id='out'></div><script>const second = document.getElementById('second'); const before = second.form; document.getElementById('mode').remove(); document.getElementById('out').textContent = before.getAttribute('id') + ':' + String(second.form);</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("option.form reflection should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "owner:null");
}

#[test]
fn session_reflects_option_disabled_through_inline_script_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><select id='mode'><option id='second' value='b'>B</option></select><div id='out'></div><script>const second = document.getElementById('second'); const before = second.disabled; second.disabled = true; const afterSet = second.disabled; second.disabled = false; const afterClear = second.disabled; document.getElementById('out').textContent = String(before) + ':' + String(afterSet) + ':' + String(afterClear) + ':' + String(document.querySelector('option:disabled'));</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("option.disabled reflection should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "false:true:false:null"
    );
}

#[test]
fn session_reflects_form_control_disabled_through_inline_script_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><input id='name'><button id='action'>Go</button><div id='out'></div><script>const input = document.getElementById('name'); const button = document.getElementById('action'); const before = String(input.disabled) + '|' + String(button.disabled) + '|' + String(document.querySelectorAll(':disabled').length) + '|' + String(document.querySelectorAll(':enabled').length); input.disabled = true; button.disabled = true; const afterSet = String(input.disabled) + '|' + String(button.disabled) + '|' + String(document.querySelectorAll(':disabled').length) + '|' + String(document.querySelectorAll(':enabled').length); input.disabled = false; button.disabled = false; const afterClear = String(input.disabled) + '|' + String(button.disabled) + '|' + String(document.querySelectorAll(':disabled').length) + '|' + String(document.querySelectorAll(':enabled').length); document.getElementById('out').textContent = before + ';' + afterSet + ';' + afterClear;</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("disabled reflection should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "false|false|0|2;true|true|2|0;false|false|0|2"
    );
}

#[test]
fn session_reflects_option_label_through_inline_script_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><select id='mode'><option id='second' value='b'>B</option></select><div id='out'></div><script>const second = document.getElementById('second'); const before = second.label; second.label = 'Bee'; const afterSet = second.label; document.getElementById('out').textContent = String(before) + ':' + String(afterSet) + ':' + String(second.textContent) + ':' + String(document.querySelector('option[label=Bee]').getAttribute('id'));</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("option.label reflection should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "B:Bee:B:second"
    );
}

#[test]
fn session_reflects_option_text_through_inline_script_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><select id='mode'><option id='second' value='b'>B</option></select><div id='out'></div><script>const second = document.getElementById('second'); const before = second.text; second.text = 'Bee'; const afterSet = second.text; document.getElementById('out').textContent = String(before) + ':' + String(afterSet) + ':' + String(second.textContent) + ':' + String(second.label);</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("option.text reflection should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "B:Bee:Bee:Bee");
}

#[test]
fn session_reflects_select_multiple_through_inline_script_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><select id='mode'><option id='second' value='b'>B</option></select><div id='out'></div><script>const select = document.getElementById('mode'); const before = select.multiple; select.multiple = true; const afterSet = select.multiple; const afterSetAttr = select.getAttribute('multiple'); select.multiple = false; const afterClear = select.multiple; document.getElementById('out').textContent = String(before) + ':' + String(afterSet) + ':' + String(afterSetAttr) + ':' + String(afterClear);</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("select.multiple reflection should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "false:true::false"
    );
}

#[test]
fn session_resolves_input_and_button_type_through_inline_script_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><input id='field'><button id='action'></button><div id='out'></div><script>const input = document.getElementById('field'); const button = document.getElementById('action'); const before = input.type + '|' + button.type; input.type = 'email'; button.type = 'reset'; const afterSet = input.type + '|' + button.type + '|' + input.getAttribute('type') + '|' + button.getAttribute('type'); input.type = 'bogus'; button.type = 'bogus'; const afterInvalid = input.type + '|' + button.type + '|' + input.getAttribute('type') + '|' + button.getAttribute('type'); document.getElementById('out').textContent = before + ';' + afterSet + ';' + afterInvalid;</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("input.type and button.type reflection should remain wired through Session");

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
fn session_reflects_select_size_through_inline_script_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><select id='mode'></select><div id='out'></div><script>const select = document.getElementById('mode'); const before = select.size; select.size = 3; const afterSet = select.size; const afterSetAttr = select.getAttribute('size'); select.size = 0; const afterZero = select.size; const afterZeroAttr = select.getAttribute('size'); document.getElementById('out').textContent = String(before) + ':' + String(afterSet) + ':' + String(afterSetAttr) + ':' + String(afterZero) + ':' + String(afterZeroAttr);</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("select.size reflection should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "0:3:3:0:0");
}

#[test]
fn session_reflects_select_required_through_inline_script_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><select id='mode'></select><div id='out'></div><script>const select = document.getElementById('mode'); const before = select.required; select.required = true; const afterSet = select.required; const afterSetAttr = select.getAttribute('required'); select.required = false; const afterClear = select.required; document.getElementById('out').textContent = String(before) + ':' + String(afterSet) + ':' + String(afterSetAttr) + ':' + String(afterClear) + ':' + String(document.querySelector('select:required'));</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("select.required reflection should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "false:true::false:null"
    );
}

#[test]
fn session_reflects_form_no_validate_through_inline_script_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><form id='signup'><button id='submit'>Submit</button><input id='field'></form><div id='out'></div><script>const form = document.getElementById('signup'); const button = document.getElementById('submit'); const field = document.getElementById('field'); const before = String(form.noValidate) + '|' + String(button.formNoValidate) + '|' + String(field.formNoValidate); form.noValidate = true; button.formNoValidate = true; field.formNoValidate = true; const afterSet = String(form.noValidate) + '|' + String(button.formNoValidate) + '|' + String(field.formNoValidate) + '|' + String(form.getAttribute('novalidate')) + '|' + String(button.getAttribute('formnovalidate')) + '|' + String(field.getAttribute('formnovalidate')); form.noValidate = false; button.formNoValidate = false; field.formNoValidate = false; const afterClear = String(form.noValidate) + '|' + String(button.formNoValidate) + '|' + String(field.formNoValidate) + '|' + String(form.hasAttribute('novalidate')) + '|' + String(button.hasAttribute('formnovalidate')) + '|' + String(field.hasAttribute('formnovalidate')); document.getElementById('out').textContent = before + ';' + afterSet + ';' + afterClear;</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("form noValidate reflection should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "false|false|false;true|true|true|||;false|false|false|false|false|false"
    );
}

#[test]
fn session_reflects_form_submission_metadata_through_inline_script_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><form id='signup' action='/submit?from=form#frag' method='BOGUS' enctype='BOGUS'><button id='submit'>Submit</button><input id='field' type='submit' formaction='/field-preview?x=1#field'></form><div id='out'></div><script>const form = document.getElementById('signup'); const button = document.getElementById('submit'); const field = document.getElementById('field'); const before = String(form.action) + '|' + String(button.formAction) + '|' + String(field.formAction) + '|' + String(form.method) + '|' + String(form.enctype) + '|' + String(form.target) + '|' + String(button.formMethod) + '|' + String(button.formEnctype) + '|' + String(button.formTarget) + '|' + String(field.formMethod) + '|' + String(field.formEnctype) + '|' + String(field.formTarget); form.action = '/override-form?x=1#form'; form.method = 'Dialog'; form.enctype = 'Multipart/Form-Data'; form.target = '_blank'; button.formAction = '/override-button?button=1#button'; button.formMethod = 'PoSt'; button.formEnctype = 'Text/Plain'; button.formTarget = 'preview'; field.formAction = '/override-field?field=1#field'; field.formMethod = 'GET'; field.formEnctype = 'application/x-www-form-urlencoded'; field.formTarget = 'field-preview'; const afterSet = String(form.action) + '|' + String(button.formAction) + '|' + String(field.formAction) + '|' + String(form.method) + '|' + String(form.enctype) + '|' + String(form.target) + '|' + String(button.formMethod) + '|' + String(button.formEnctype) + '|' + String(button.formTarget) + '|' + String(field.formMethod) + '|' + String(field.formEnctype) + '|' + String(field.formTarget) + '|' + String(form.getAttribute('action')) + '|' + String(form.getAttribute('method')) + '|' + String(form.getAttribute('enctype')) + '|' + String(form.getAttribute('target')) + '|' + String(button.getAttribute('formaction')) + '|' + String(button.getAttribute('formmethod')) + '|' + String(button.getAttribute('formenctype')) + '|' + String(button.getAttribute('formtarget')) + '|' + String(field.getAttribute('formaction')) + '|' + String(field.getAttribute('formmethod')) + '|' + String(field.getAttribute('formenctype')) + '|' + String(field.getAttribute('formtarget')); document.getElementById('out').textContent = before + ';' + afterSet;</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("form submission metadata reflection should remain wired through Session");

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
            "<main id='root'><div id='box'></div></main><script>document.getElementById('box').method;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-form submission metadata getter should fail explicitly");

    assert!(session.to_string().contains("method"));
}

#[test]
fn session_reflects_input_textarea_readonly_through_inline_script_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><input id='name' value='Ada'><textarea id='bio'>Hello</textarea><div id='out'></div><script>const input = document.getElementById('name'); const textarea = document.getElementById('bio'); const before = String(input.readOnly) + '|' + String(textarea.readOnly); input.readOnly = true; textarea.readOnly = true; const afterSet = String(input.readOnly) + '|' + String(textarea.readOnly); const afterAttr = String(input.hasAttribute('readonly')) + '|' + String(textarea.hasAttribute('readonly')); const afterReadOnly = String(input.matches(':read-only')) + '|' + String(textarea.matches(':read-only')); input.readOnly = false; textarea.readOnly = false; const afterClear = String(input.readOnly) + '|' + String(textarea.readOnly); const afterClearReadOnly = String(input.matches(':read-only')) + '|' + String(textarea.matches(':read-only')); document.getElementById('out').textContent = before + ';' + afterSet + ';' + afterAttr + ';' + afterReadOnly + ';' + afterClear + ';' + afterClearReadOnly;</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("readOnly reflection should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "false|false;true|true;true|true;true|true;false|false;false|false"
    );
}

#[test]
fn session_reflects_input_textarea_placeholder_through_inline_script_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><input id='name' placeholder='Name'><textarea id='bio' placeholder='Bio'></textarea><div id='out'></div><script>const input = document.getElementById('name'); const textarea = document.getElementById('bio'); const before = String(input.placeholder) + '|' + String(textarea.placeholder) + '|' + String(document.querySelectorAll(':placeholder-shown').length); input.placeholder = 'Full name'; textarea.placeholder = 'Biography'; const afterSet = String(input.placeholder) + '|' + String(textarea.placeholder) + '|' + String(input.getAttribute('placeholder')) + '|' + String(textarea.getAttribute('placeholder')) + '|' + String(document.querySelectorAll(':placeholder-shown').length); input.value = 'Alice'; textarea.value = 'Bio text'; const afterValue = String(document.querySelectorAll(':placeholder-shown').length); document.getElementById('out').textContent = before + ';' + afterSet + ';' + afterValue;</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("placeholder reflection should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "Name|Bio|2;Full name|Biography|Full name|Biography|2;0"
    );
}

#[test]
fn session_reflects_blank_pseudo_class_through_inline_script_regression() {
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
fn session_reflects_input_textarea_autofocus_through_inline_script_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><input id='name'><textarea id='bio'></textarea><div id='out'></div><script>const input = document.getElementById('name'); const textarea = document.getElementById('bio'); const before = String(input.autofocus) + '|' + String(textarea.autofocus) + '|' + String(document.querySelectorAll('[autofocus]').length); input.autofocus = true; textarea.autofocus = true; const afterSet = String(input.autofocus) + '|' + String(textarea.autofocus) + '|' + String(document.querySelectorAll('[autofocus]').length); input.autofocus = false; textarea.autofocus = false; const afterClear = String(input.autofocus) + '|' + String(textarea.autofocus) + '|' + String(document.querySelectorAll('[autofocus]').length); document.getElementById('out').textContent = before + ';' + afterSet + ';' + afterClear;</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("autofocus reflection should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "false|false|0;true|true|2;false|false|0"
    );
}

#[test]
fn session_reflects_input_textarea_autocomplete_through_inline_script_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><input id='name'><textarea id='bio'></textarea><div id='out'></div><script>const input = document.getElementById('name'); const textarea = document.getElementById('bio'); const before = String(input.autocomplete) + '|' + String(textarea.autocomplete); input.autocomplete = 'email'; textarea.autocomplete = 'off'; const afterSet = String(input.autocomplete) + '|' + String(textarea.autocomplete); const afterAttr = String(input.getAttribute('autocomplete')) + '|' + String(textarea.getAttribute('autocomplete')); input.autocomplete = ''; textarea.autocomplete = ''; const afterClear = String(input.autocomplete) + '|' + String(textarea.autocomplete) + '|' + String(input.hasAttribute('autocomplete')) + '|' + String(textarea.hasAttribute('autocomplete')); document.getElementById('out').textContent = before + ';' + afterSet + ';' + afterAttr + ';' + afterClear;</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("autocomplete reflection should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "|;email|off;email|off;||true|true"
    );
}

#[test]
fn session_reflects_input_file_accept_and_multiple_through_inline_script_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><input id='upload' type='file'><div id='out'></div><script>const upload = document.getElementById('upload'); const before = String(upload.accept) + '|' + String(upload.multiple) + '|' + String(upload.hasAttribute('accept')) + '|' + String(upload.hasAttribute('multiple')); upload.accept = 'image/*'; upload.multiple = true; const afterSet = String(upload.accept) + '|' + String(upload.multiple) + '|' + String(upload.getAttribute('accept')) + '|' + String(upload.hasAttribute('multiple')); upload.accept = ''; upload.multiple = false; const afterClear = String(upload.accept) + '|' + String(upload.multiple) + '|' + String(upload.getAttribute('accept')) + '|' + String(upload.hasAttribute('multiple')); document.getElementById('out').textContent = before + ';' + afterSet + ';' + afterClear;</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("file input accept/multiple reflection should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "|false|false|false;image/*|true|image/*|true;|false||false"
    );
    assert_eq!(
        session
            .dom()
            .select("#upload[accept='image/*']")
            .unwrap()
            .len(),
        0
    );
    assert_eq!(session.dom().select("#upload[multiple]").unwrap().len(), 0);
}

#[test]
fn session_reflects_input_textarea_length_through_inline_script_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><input id='name' minlength='2' maxlength='4' value='Ada'><textarea id='bio' minlength='2' maxlength='4'>Bio</textarea><div id='out'></div><script>const input = document.getElementById('name'); const textarea = document.getElementById('bio'); const before = String(input.minLength) + '|' + String(input.maxLength) + '|' + String(textarea.minLength) + '|' + String(textarea.maxLength) + '|' + String(document.querySelectorAll(':invalid').length); input.minLength = 4; textarea.maxLength = 2; const afterSet = String(input.minLength) + '|' + String(input.maxLength) + '|' + String(textarea.minLength) + '|' + String(textarea.maxLength) + '|' + String(document.querySelectorAll(':invalid').length); input.value = 'Jane'; textarea.value = 'No'; const afterValue = String(document.querySelectorAll(':invalid').length) + ':' + String(document.querySelectorAll(':valid').length); document.getElementById('out').textContent = before + ';' + afterSet + ';' + afterValue;</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("length reflection should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "2|4|2|4|0;4|4|2|2|2;0:2"
    );
}

#[test]
fn session_reflects_input_range_bounds_through_inline_script_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><input id='low' type='number' value='1'><input id='high' type='number' value='4'><div id='out'></div><script>const low = document.getElementById('low'); const high = document.getElementById('high'); const before = String(low.min) + '|' + String(low.max) + '|' + String(high.min) + '|' + String(high.max) + '|' + String(document.querySelectorAll(':in-range').length) + '|' + String(document.querySelectorAll(':out-of-range').length); low.min = 2; low.max = 6; high.min = 2; high.max = 6; const afterSet = String(low.min) + '|' + String(low.max) + '|' + String(high.min) + '|' + String(high.max) + '|' + String(document.querySelectorAll(':in-range').length) + '|' + String(document.querySelectorAll(':out-of-range').length) + '|' + document.querySelectorAll(':in-range').item(0).id + '|' + document.querySelectorAll(':out-of-range').item(0).id; low.removeAttribute('min'); low.removeAttribute('max'); high.removeAttribute('min'); high.removeAttribute('max'); const afterClear = String(low.min) + '|' + String(low.max) + '|' + String(high.min) + '|' + String(high.max) + '|' + String(document.querySelectorAll(':in-range').length) + '|' + String(document.querySelectorAll(':out-of-range').length); document.getElementById('out').textContent = before + ';' + afterSet + ';' + afterClear;</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("range bounds reflection should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "||||0|0;2|6|2|6|1|1|high|low;||||0|0"
    );
}

#[test]
fn session_reflects_input_pattern_through_inline_script_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><input id='name' pattern='A[a-z]+' value='Ada'><div id='out'></div><script>const input = document.getElementById('name'); const before = String(input.pattern) + '|' + String(document.querySelectorAll(':invalid').length); input.pattern = 'B[a-z]+'; const afterSet = String(input.pattern) + '|' + String(input.getAttribute('pattern')) + '|' + String(document.querySelectorAll(':invalid').length); input.value = 'Bob'; const afterValue = String(document.querySelectorAll(':invalid').length) + ':' + String(document.querySelectorAll(':valid').length); document.getElementById('out').textContent = before + ';' + afterSet + ';' + afterValue;</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("pattern reflection should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "A[a-z]+|0;B[a-z]+|B[a-z]+|1;0:1"
    );
}

#[test]
fn session_rejects_non_form_control_length_access_regression() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='wrapper'><button id='button'>Button</button></div><script>document.getElementById('button').minLength = 1;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-form-control minLength access should fail explicitly");

    assert!(error.to_string().contains("minLength"));
}

#[test]
fn session_rejects_non_input_pattern_access_regression() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='wrapper'><button id='button'>Button</button></div><script>document.getElementById('button').pattern = 'abc';</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-input pattern access should fail explicitly");

    assert!(error.to_string().contains("pattern"));
}

#[test]
fn session_rejects_non_form_control_disabled_access_regression() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='wrapper'><div id='box'></div></div><script>document.getElementById('box').disabled = true;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-form-control disabled access should fail explicitly");

    assert!(error.to_string().contains("disabled"));
}

#[test]
fn session_rejects_non_input_range_bounds_access_regression() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='wrapper'><button id='button'>Button</button></div><script>document.getElementById('button').min = 1;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-input min access should fail explicitly");

    assert!(error.to_string().contains("min"));
}

#[test]
fn session_reflects_class_views_through_inline_script_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><button id='button' class='base' data-kind='App'>First</button><div id='out'></div><script>const button = document.getElementById('button'); button.className = 'primary secondary'; const before = button.classList.length; const contains = button.classList.contains('primary'); button.classList.add('tertiary'); button.classList.remove('secondary'); const toggled = button.classList.toggle('active'); button.dataset.userId = '42'; const out = document.getElementById('out'); out.textContent = button.className + ':' + String(before) + ':' + String(contains) + ':' + String(toggled) + ':' + button.dataset.kind + ':' + button.dataset.userId + ':' + String(button.classList) + ':' + button.classList.toString() + ':' + String(button.classList.item(1)) + ':' + String(button.classList.keys().next().value) + ':' + String(button.classList.values().next().value) + ':' + String(button.classList.entries().next().value.index) + ':' + String(button.classList.entries().next().value.value) + ':' + String(button.dataset); button.classList.forEach((token, index, list) => { out.textContent += '|F' + String(index) + ':' + token + ':' + String(list.length); });</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("class views should remain wired through Session");

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
fn session_reflects_classlist_value_through_inline_script_regression() {
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
}

#[test]
fn session_reflects_classlist_replace_through_inline_script_regression() {
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
fn session_resolves_document_forms_named_properties_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><form id='signup' name='signup'>Signup</form><form id='login' name='login'>Login</form></div><div id='out'></div><script>const forms = document.forms; const signup = forms.signup; const login = forms.login; document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = signup.textContent + ':' + login.textContent + ':' + String(forms.missing);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("document.forms named properties should remain available");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "Signup:Login:undefined"
    );
}

#[test]
fn session_resolves_document_forms_iterator_helpers_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><form id='first-form' name='first-form'>First</form><form id='second-form' name='second-form'>Second</form></div><div id='out'></div><script>const forms = document.forms; const keys = forms.keys(); const values = forms.values(); const entries = forms.entries(); const firstKey = keys.next(); const firstValue = values.next(); const firstEntry = entries.next(); let out = ''; forms.forEach((element, index, list) => { out += String(index) + ':' + element.getAttribute('id') + ':' + String(list.length) + ';'; }); document.getElementById('out').textContent = String(firstKey.value) + ':' + firstValue.value.getAttribute('id') + ':' + String(firstEntry.value.index) + ':' + firstEntry.value.value.getAttribute('id') + ':' + out;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("document.forms iterator helpers should remain available");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "0:first-form:0:first-form:0:first-form:2;1:second-form:2;"
    );
}

#[test]
fn session_resolves_named_node_map_for_each_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><div id='box' data-role='menu' data-state='azure'></div><div id='out'></div><script>const attrs = document.getElementById('box').attributes; let out = ''; attrs.forEach((attr, index, list) => { out += String(index) + ':' + attr.name + ':' + attr.value + ':' + String(list.length) + ';'; }); document.getElementById('out').textContent = out;</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("NamedNodeMap.forEach should remain available");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "0:data-role:menu:3;1:data-state:azure:3;2:id:box:3;"
    );
}

#[test]
fn session_resolves_named_node_map_iterator_helpers_exhaustion_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><div id='box' data-role='menu' data-state='azure'></div><div id='out'></div><script>const attrs = document.getElementById('box').attributes; const keys = attrs.keys(); const values = attrs.values(); const entries = attrs.entries(); const firstKey = keys.next(); const firstValue = values.next(); const firstEntry = entries.next(); document.getElementById('out').textContent = String(attrs) + ':' + String(firstKey.value) + ':' + firstValue.value.name + ':' + String(firstEntry.value.index) + ':' + firstEntry.value.value.name;</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("NamedNodeMap iterator helpers should remain available");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "[object NamedNodeMap]:0:data-role:0:data-role"
    );
}

#[test]
fn session_resolves_named_node_map_iterator_helpers_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><div id='box' data-role='menu' data-state='azure'></div><div id='out'></div><script>const attrs = document.getElementById('box').attributes; const keys = attrs.keys(); const values = attrs.values(); const entries = attrs.entries(); const firstKey = keys.next(); const secondKey = keys.next(); const thirdKey = keys.next(); const firstValue = values.next(); const secondValue = values.next(); const thirdValue = values.next(); const firstEntry = entries.next(); const secondEntry = entries.next(); const thirdEntry = entries.next(); document.getElementById('out').textContent = String(attrs) + ':' + String(firstKey.value) + ':' + String(secondKey.value) + ':' + String(thirdKey.done) + ':' + firstValue.value.name + ':' + secondValue.value.name + ':' + String(thirdValue.done) + ':' + String(firstEntry.value.index) + ':' + firstEntry.value.value.name + ':' + String(secondEntry.value.index) + ':' + secondEntry.value.value.name + ':' + String(thirdEntry.done);</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("NamedNodeMap iterator helpers should remain available");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "[object NamedNodeMap]:0:1:false:data-role:data-state:false:0:data-role:1:data-state:false"
    );
}

#[test]
fn session_resolves_attribute_specified_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><div id='box' data-role='menu'></div><div id='out'></div><script>const detached = document.createAttribute('data-state'); const attached = document.getElementById('box').getAttributeNode('data-role'); document.getElementById('out').textContent = String(detached.specified) + ':' + String(attached.specified);</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("Attr.specified should remain available");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "true:true");
}

#[test]
fn session_resolves_attribute_is_id_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><div id='box'></div><div id='out'></div><script>const detached = document.createAttribute('data-state'); const attached = document.getElementById('box').getAttributeNode('id'); document.getElementById('out').textContent = String(detached.isId) + ':' + String(attached.isId);</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("Attr.isId should remain available");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "false:true");
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
fn session_resolves_radio_node_list_iterator_helpers_regression() {
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
fn session_resolves_document_scripts_iterator_helpers_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><script id='first-script'></script></div><div id='out'></div><script id='current-script'>const out = document.getElementById('out'); const scripts = document.scripts; const keys = scripts.keys(); const values = scripts.values(); const entries = scripts.entries(); const firstKey = keys.next(); const firstValue = values.next(); const firstEntry = entries.next(); let serial = ''; scripts.forEach((element, index, list) => { serial += String(index) + ':' + element.getAttribute('id') + ':' + String(list.length) + ';'; }); document.getElementById('root').textContent = 'gone'; out.textContent = String(firstKey.value) + ':' + firstValue.value.getAttribute('id') + ':' + String(firstEntry.value.index) + ':' + firstEntry.value.value.getAttribute('id') + ':' + serial + ':' + String(scripts.length);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("document.scripts iterator helpers should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "0:first-script:0:first-script:0:first-script:2;1:current-script:2;:1"
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
fn session_resolves_document_style_sheets_for_each_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><style id='first-style'>.primary { color: red; }</style><link id='first-link' rel='stylesheet' href='a.css'></div><div id='out'></div><script>const out = document.getElementById('out'); const sheets = document.styleSheets; let value = ''; sheets.forEach((sheet, index, list) => { value += String(index) + ':' + String(sheet) + ':' + String(list.length) + ';'; }); out.textContent = value;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("document.styleSheets.forEach should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "0:[object CSSStyleSheet]:2;1:[object CSSStyleSheet]:2;"
    );
}

#[test]
fn session_resolves_document_create_element_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'></main><div id='out'></div><script>const root = document.getElementById('root'); const created = document.createElement('span'); created.setAttribute('id', 'created'); created.textContent = 'Hello'; root.appendChild(created); document.getElementById('out').textContent = String(root.children.length) + ':' + root.children.namedItem('created').textContent + ':' + created.ownerDocument.documentElement.getAttribute('id');</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("document.createElement should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "1:Hello:root");
}

#[test]
fn session_resolves_document_create_text_node_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'></main><div id='out'></div><script>const root = document.getElementById('root'); const text = document.createTextNode('Hello'); root.appendChild(text); document.getElementById('out').textContent = root.textContent + ':' + String(text.nodeType) + ':' + text.nodeName;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("document.createTextNode should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "Hello:3:#text");
}

#[test]
fn session_resolves_document_create_comment_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'></main><div id='out'></div><script>const root = document.getElementById('root'); const comment = document.createComment('Hello'); root.appendChild(comment); document.getElementById('out').textContent = root.textContent + ':' + String(comment.nodeType) + ':' + comment.nodeName + ':' + comment.textContent + ':' + String(root.childNodes.length);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("document.createComment should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        ":8:#comment::1"
    );
}

#[test]
fn session_resolves_document_create_document_fragment_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'></main><div id='out'></div><script>const root = document.getElementById('root'); const frag = document.createDocumentFragment(); frag.appendChild(document.createTextNode('Hello')); const returned = root.appendChild(frag); document.getElementById('out').textContent = String(returned) + ':' + String(frag.childNodes.length) + ':' + root.textContent + ':' + String(root.childNodes.length);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("document.createDocumentFragment should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "[object DocumentFragment]:0:Hello:1"
    );
}

#[test]
fn session_resolves_node_equality_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>const left = document.createElement('div'); left.appendChild(document.createTextNode('Hello')); const right = document.createElement('div'); right.appendChild(document.createTextNode('Hello')); const fragLeft = document.createDocumentFragment(); fragLeft.appendChild(document.createTextNode('Hello')); const fragRight = document.createDocumentFragment(); fragRight.appendChild(document.createTextNode('Hello')); document.getElementById('out').textContent = String(left.isSameNode(right)) + ':' + String(left.isEqualNode(right)) + ':' + String(fragLeft.isSameNode(fragRight)) + ':' + String(fragLeft.isEqualNode(fragRight));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("isEqualNode should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "false:true:false:true"
    );
}

#[test]
fn session_resolves_document_import_node_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'></main><div id='out'></div><script>const root = document.getElementById('root'); const source = document.createDocumentFragment(); source.appendChild(document.createTextNode('Hello')); const imported = document.importNode(source, true); const returned = root.appendChild(imported); document.getElementById('out').textContent = String(returned) + ':' + String(imported.childNodes.length) + ':' + root.textContent + ':' + String(root.childNodes.length);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("document.importNode should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "[object DocumentFragment]:0:Hello:1"
    );
}

#[test]
fn session_resolves_node_normalize_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'>First</main><div id='out'></div><script>const root = document.getElementById('root'); root.appendChild(document.createTextNode('Second')); root.appendChild(document.createTextNode('Third')); root.normalize(); document.getElementById('out').textContent = String(root.childNodes.length) + ':' + String(root.firstChild.nodeType) + ':' + root.textContent;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("Node.normalize should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "1:3:FirstSecondThird"
    );
}

#[test]
fn session_resolves_node_remove_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'></main><div id='out'></div><script>const root = document.getElementById('root'); const child = document.createTextNode('Hello'); root.appendChild(child); child.remove(); document.getElementById('out').textContent = String(child.parentNode) + ':' + String(root.childNodes.length);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("Node.remove should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "null:0");
}

#[test]
fn session_rejects_element_remove_child_regression() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><span id='child'>Child</span></main><div id='out'></div><script>const root = document.getElementById('root'); const orphan = document.createElement('span'); root.removeChild(orphan);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("Element.removeChild should reject children from another parent");

    let message = error.to_string();
    assert!(message.contains("removeChild()"));
    assert!(message.contains("expects the child to belong to the parent"));
}

#[test]
fn session_resolves_node_clone_node_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'></main><div id='out'></div><script>const root = document.getElementById('root'); const child = document.createTextNode('Hello'); root.appendChild(child); const clone = root.cloneNode(true); document.getElementById('out').textContent = String(clone) + ':' + String(clone.childNodes.length) + ':' + clone.childNodes.item(0).nodeName + ':' + String(clone.childNodes.item(0).nodeType) + ':' + clone.childNodes.item(0).textContent + ':' + String(root.childNodes.length);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("Node.cloneNode should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "[object Element]:1:#text:3:Hello:1"
    );
}

#[test]
fn session_resolves_node_replace_with_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'>First<span id='second'>Second</span></main><div id='out'></div><script>const root = document.getElementById('root'); const first = root.childNodes.item(0); first.replaceWith(document.createTextNode('Alpha')); document.getElementById('out').textContent = String(root.childNodes.length) + ':' + String(root.childNodes.item(0).nodeType) + ':' + root.childNodes.item(0).textContent + ':' + root.childNodes.item(1).textContent + ':' + String(first.parentNode);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("Node.replaceWith should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "2:3:Alpha:Second:null"
    );
}

#[test]
fn session_resolves_node_contains_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><span id='child'>Child</span></main><div id='out'></div><script>const root = document.getElementById('root'); const child = document.getElementById('child'); const detached = document.createElement('div'); document.getElementById('out').textContent = String(document.contains(root)) + ':' + String(root.contains(child)) + ':' + String(child.contains(root)) + ':' + String(detached.contains(detached)) + ':' + String(detached.contains(root)) + ':' + String(root.contains(null));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("contains should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "true:true:false:true:false:false"
    );
}

#[test]
fn session_resolves_has_child_nodes_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><span id='child'>Child</span></main><template id='tpl'><span id='inner'>Inner</span></template><div id='out'></div><script>const root = document.getElementById('root'); const child = document.getElementById('child'); const tpl = document.getElementById('tpl'); document.getElementById('out').textContent = String(document.hasChildNodes()) + ':' + String(root.hasChildNodes()) + ':' + String(child.hasChildNodes()) + ':' + String(tpl.content.hasChildNodes());</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("hasChildNodes should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "true:true:true:true"
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
fn session_resolves_document_applets_iterator_helpers_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><applet id='first-applet' name='first-applet'>First</applet><applet id='second-applet' name='second-applet'>Second</applet></div><div id='out'></div><script>const applets = document.applets; const keys = applets.keys(); const values = applets.values(); const entries = applets.entries(); const firstKey = keys.next(); const firstValue = values.next(); const firstEntry = entries.next(); let out = ''; applets.forEach((element, index, list) => { out += String(index) + ':' + element.textContent + ':' + String(list.length) + ';'; }); document.getElementById('out').textContent = String(firstKey.value) + ':' + firstValue.value.textContent + ':' + String(firstEntry.value.index) + ':' + firstEntry.value.value.textContent + ':' + out;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("document.applets iterator helpers should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "0:First:0:First:0:First:2;1:Second:2;"
    );
}

#[test]
fn session_resolves_element_tag_name_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'></main><div id='out'></div><script>document.getElementById('out').textContent = document.getElementById('root').tagName;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("element.tagName should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "main");
}

#[test]
fn session_resolves_element_local_name_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'></main><div id='out'></div><script>document.getElementById('out').textContent = document.getElementById('root').localName;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("element.localName should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "main");
}

#[test]
fn session_resolves_element_namespace_uri_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'></main><div id='out'></div><script>document.getElementById('out').textContent = document.getElementById('root').namespaceURI;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("element.namespaceURI should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "http://www.w3.org/1999/xhtml"
    );
}

#[test]
fn session_resolves_create_element_ns_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some("<main id='root'></main><div id='out'></div><script>const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg'); svg.setAttribute('viewBox', '0 0 10 10'); document.getElementById('root').appendChild(svg); document.getElementById('out').textContent = svg.namespaceURI + '|' + svg.localName + '|' + svg.outerHTML;</script>".to_string()),
        local_storage: BTreeMap::new(),
    })
    .expect("document.createElementNS should remain wired through Session");

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
fn session_resolves_element_name_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root' name='root'></main><div id='out'></div><script>const root = document.getElementById('root'); const before = document.getElementsByName('root').length; root.name = 'next'; const next = document.getElementsByName('next'); document.getElementById('out').textContent = String(before) + ':' + String(document.getElementsByName('root').length) + ':' + String(next.length) + ':' + next.item(0).getAttribute('id');</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("element.name should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "1:0:1:root");
}

#[test]
fn session_rejects_element_tag_name_assignment_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'></main><script>document.getElementById('root').tagName = 'MAIN';</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("element.tagName should be read-only");

    let message = error.to_string();
    assert!(message.contains("unsupported assignment target"));
    assert!(message.contains("tagName"));
}

#[test]
fn session_rejects_element_namespace_uri_assignment_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'></main><script>document.getElementById('root').namespaceURI = 'http://www.w3.org/1999/xhtml';</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("element.namespaceURI should be read-only");

    let message = error.to_string();
    assert!(message.contains("unsupported assignment target"));
    assert!(message.contains("namespaceURI"));
}

#[test]
fn session_rejects_element_local_name_assignment_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'></main><script>document.getElementById('root').localName = 'MAIN';</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("element.localName should be read-only");

    let message = error.to_string();
    assert!(message.contains("unsupported assignment target"));
    assert!(message.contains("localName"));
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
fn session_resolves_template_content_query_selector_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<template id='tpl'><span class='primary' id='first'>First</span><span class='primary' id='second'>Second</span></template><div id='out'></div><script>const tpl = document.getElementById('tpl'); const content = tpl.content; const first = content.querySelector('.primary'); const all = content.querySelectorAll('.primary'); document.getElementById('out').textContent = String(content) + ':' + first.textContent + ':' + String(all.length) + ':' + all.item(0).textContent + ':' + all.item(1).textContent + ':' + String(all.item(2));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("template content query selectors should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "[object DocumentFragment]:First:2:First:Second:null"
    );
}

#[test]
fn session_resolves_template_content_get_element_by_id_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<template id='tpl'><span id='foo,bar'>First</span></template><div id='out'></div><script>const tpl = document.getElementById('tpl'); const content = tpl.content; const hit = content.getElementById('foo,bar'); document.getElementById('out').textContent = String(hit) + ':' + hit.textContent;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("template content getElementById should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "[object Element]:First"
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
fn session_resolves_template_content_text_content_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<template id='tpl'><span id='inner'>Inner</span></template><div id='out'></div><script>const content = document.getElementById('tpl').content; const before = content.textContent; content.textContent = 'Updated'; document.getElementById('out').textContent = before + ':' + content.textContent + ':' + content.innerHTML;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("template content textContent should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "Inner:Updated:Updated"
    );
}

#[test]
fn session_resolves_template_content_fragment_reflection_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<template id='tpl'><span id='inner'>Inner</span></template><div id='out'></div><script>const content = document.getElementById('tpl').content; document.getElementById('out').textContent = String(content.nodeType) + ':' + content.nodeName + ':' + String(content.parentNode) + ':' + String(content.ownerDocument);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("template content fragment reflection should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "11:#document-fragment:null:[object Document]"
    );
}

#[test]
fn session_resolves_template_content_clone_node_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<template id='tpl'><span id='inner'>Inner</span></template><div id='out'></div><script>const content = document.getElementById('tpl').content; const deep = content.cloneNode(true); const shallow = content.cloneNode(); document.getElementById('out').textContent = String(deep) + ':' + String(deep.childNodes.length) + ':' + deep.childNodes.item(0).nodeName + ':' + deep.childNodes.item(0).textContent + ':' + String(shallow.childNodes.length);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("template content cloneNode should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "[object DocumentFragment]:1:span:Inner:0"
    );
}

#[test]
fn session_resolves_template_content_append_child_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<template id='tpl'><span id='inner'>Inner</span></template><div id='out'></div><script>const content = document.getElementById('tpl').content; const clone = content.cloneNode(true); const child = clone.childNodes.item(0); content.appendChild(child); document.getElementById('out').textContent = String(content.childNodes.length) + ':' + content.childNodes.item(1).textContent + ':' + String(clone.childNodes.length);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("template content appendChild should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "2:Inner:0");
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
fn session_resolves_window_children_iterator_helpers_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><span id='first'>First</span><span id='second'>Second</span></div><div id='out'></div><script id='script'>const children = document.defaultView.children; const keys = children.keys(); const values = children.values(); const entries = children.entries(); const firstKey = keys.next(); const firstValue = values.next(); const firstEntry = entries.next(); let out = ''; children.forEach((element, index, list) => { out += String(index) + ':' + element.getAttribute('id') + ':' + String(list.length) + ';'; }); document.getElementById('out').textContent = String(firstKey.value) + ':' + firstValue.value.getAttribute('id') + ':' + String(firstEntry.value.index) + ':' + firstEntry.value.value.getAttribute('id') + ':' + out;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("window.children iterator helpers should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "0:root:0:root:0:root:3;1:out:3;2:script:3;"
    );
}

#[test]
fn session_resolves_owner_document_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<html><body id='body'><main id='out'></main><script>document.getElementById('out').textContent = String(document.body.ownerDocument) + ':' + String(document.body.ownerDocument.defaultView) + ':' + String(document.ownerDocument);</script></body></html>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("ownerDocument should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "[object Document]:[object Window]:null"
    );
}

#[test]
fn session_resolves_parent_node_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<html><body id='body'>Text<main id='out'></main><script>const body = document.body; const text = body.childNodes.item(0); document.getElementById('out').textContent = String(document.parentNode) + ':' + String(document.documentElement.parentNode) + ':' + String(document.documentElement.parentElement) + ':' + String(body.parentNode) + ':' + String(body.parentElement) + ':' + String(text.parentNode) + ':' + String(text.parentElement);</script></body></html>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("parentNode should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "null:[object Document]:null:[object Element]:[object Element]:[object Element]:[object Element]"
    );
}

#[test]
fn session_rejects_parent_node_assignment_regression() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<body id='body'><script>document.body.parentNode = null;</script></body>".to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("parentNode should be read-only");

    let message = error.to_string();
    assert!(message.contains("unsupported assignment target"));
    assert!(message.contains("parentNode"));
}

#[test]
fn session_resolves_first_and_last_element_child_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<html><head></head><body><div id='wrapper'><span></span><span></span></div><template id='tmpl'><span></span><span></span></template><main id='out'></main><script>const html = document.documentElement; const wrapper = document.getElementById('wrapper'); const template = document.getElementById('tmpl').content; document.getElementById('out').textContent = String(document.childElementCount) + ':' + String(document.firstElementChild) + ':' + String(document.lastElementChild) + ':' + String(html.childElementCount) + ':' + String(html.firstElementChild) + ':' + String(html.lastElementChild) + ':' + String(wrapper.childElementCount) + ':' + String(wrapper.firstElementChild) + ':' + String(wrapper.lastElementChild) + ':' + String(template.childElementCount) + ':' + String(template.firstElementChild) + ':' + String(template.lastElementChild);</script></body></html>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("first/lastElementChild should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "1:[object Element]:[object Element]:2:[object Element]:[object Element]:2:[object Element]:[object Element]:2:[object Element]:[object Element]"
    );
}

#[test]
fn session_rejects_first_element_child_assignment_regression() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<body id='body'><script>document.body.firstElementChild = null;</script></body>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("firstElementChild should be read-only");

    let message = error.to_string();
    assert!(message.contains("unsupported assignment target"));
    assert!(message.contains("firstElementChild"));
}

#[test]
fn session_rejects_window_frames_length_assignment_regression() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<iframe id='first'></iframe><script>window.frames.length = 2;</script>".to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("window.frames.length should be read-only");

    let message = error.to_string();
    assert!(message.contains("cannot assign to `length` on html collection value"));
}

#[test]
fn session_resolves_window_frames_iterator_helpers_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<iframe id='first'></iframe><iframe id='second'></iframe><div id='out'></div><script>const frames = window.frames; const keys = frames.keys(); const values = frames.values(); const entries = frames.entries(); const firstKey = keys.next(); const firstValue = values.next(); const firstEntry = entries.next(); let out = ''; frames.forEach((element, index, list) => { out += String(index) + ':' + element.getAttribute('id') + ':' + String(list.length) + ';'; }); document.getElementById('out').textContent = String(firstKey.value) + ':' + firstValue.value.getAttribute('id') + ':' + String(firstEntry.value.index) + ':' + firstEntry.value.value.getAttribute('id') + ':' + out;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("window.frames iterator helpers should resolve");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "0:first:0:first:0:first:2;1:second:2;"
    );
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
fn session_exposes_document_design_mode_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>const before = document.designMode; document.designMode = 'on'; document.getElementById('out').textContent = before + ':' + document.designMode;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("document.designMode should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "off:on");
    assert_eq!(session.document_design_mode(), "on");
}

#[test]
fn session_rejects_invalid_document_design_mode_assignment_regression() {
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
fn session_exposes_element_content_editable_getter_setter_and_is_content_editable_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><div id='parent' contenteditable='true'><span id='child' contenteditable='false'>Edit</span></div><script>const child = document.getElementById('child'); const before = child.contentEditable; const beforeState = child.isContentEditable; child.contentEditable = 'inherit'; const after = child.contentEditable; const afterState = child.isContentEditable; document.getElementById('out').textContent = before + ':' + String(beforeState) + ':' + after + ':' + String(afterState) + ':' + String(child.matches(':read-write'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("element.contentEditable should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "false:false:inherit:true:true"
    );
}

#[test]
fn session_exposes_element_translate_getter_setter_with_inheritance_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><div id='parent' translate='no'><span id='child'>Edit</span></div><script>const child = document.getElementById('child'); const before = String(child.translate); child.translate = true; const afterTrue = String(child.translate) + ':' + child.getAttribute('translate'); child.translate = false; const afterFalse = String(child.translate) + ':' + child.getAttribute('translate'); document.getElementById('out').textContent = before + ':' + afterTrue + ':' + afterFalse;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("element.translate should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "false:true:yes:false:no"
    );
}

#[test]
fn session_rejects_invalid_element_content_editable_assignment_regression() {
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
fn session_exposes_element_title_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><div id='box' title='Initial'></div><div id='out'></div><script>const box = document.getElementById('box'); const before = box.title; box.title = 'Updated'; document.getElementById('out').textContent = before + ':' + box.title + ':' + box.getAttribute('title');</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("element.title should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "Initial:Updated:Updated"
    );
}

#[test]
fn session_exposes_element_role_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><div id='box' role='button'></div><div id='out'></div><script>const box = document.getElementById('box'); const before = box.role; box.role = 'menu'; document.getElementById('out').textContent = before + ':' + box.role + ':' + box.getAttribute('role');</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("element.role should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "button:menu:menu"
    );
}

#[test]
fn session_exposes_element_tab_index_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><div id='box'></div><button id='button'></button><div id='out'></div><script>const box = document.getElementById('box'); const button = document.getElementById('button'); const before = String(box.tabIndex) + '|' + String(button.tabIndex); box.tabIndex = 4; button.tabIndex = -1; document.getElementById('out').textContent = before + ':' + String(box.tabIndex) + '|' + String(button.tabIndex) + ':' + box.getAttribute('tabindex') + '|' + button.getAttribute('tabindex');</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("element.tabIndex should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "-1|0:4|-1:4|-1"
    );
}

#[test]
fn session_exposes_element_aria_label_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><div id='box' aria-label='Initial'></div><div id='out'></div><script>const box = document.getElementById('box'); const before = box.ariaLabel; box.ariaLabel = 'Updated'; document.getElementById('out').textContent = before + ':' + box.ariaLabel + ':' + box.getAttribute('aria-label');</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("element.ariaLabel should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "Initial:Updated:Updated"
    );
}

#[test]
fn session_exposes_element_aria_description_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><div id='box' aria-description='Initial'></div><div id='out'></div><script>const box = document.getElementById('box'); const before = box.ariaDescription; box.ariaDescription = 'Updated'; document.getElementById('out').textContent = before + ':' + box.ariaDescription + ':' + box.getAttribute('aria-description');</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("element.ariaDescription should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "Initial:Updated:Updated"
    );
}

#[test]
fn session_exposes_element_aria_role_description_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><div id='box' aria-roledescription='Initial'></div><div id='out'></div><script>const box = document.getElementById('box'); const before = box.ariaRoleDescription; box.ariaRoleDescription = 'Updated'; document.getElementById('out').textContent = before + ':' + box.ariaRoleDescription + ':' + box.getAttribute('aria-roledescription');</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("element.ariaRoleDescription should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "Initial:Updated:Updated"
    );
}

#[test]
fn session_exposes_element_aria_hidden_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><div id='box' aria-hidden='true'></div><div id='out'></div><script>const box = document.getElementById('box'); const before = box.ariaHidden; box.ariaHidden = false; document.getElementById('out').textContent = before + ':' + box.ariaHidden + ':' + box.getAttribute('aria-hidden');</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("element.ariaHidden should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "true:false:false"
    );
}

#[test]
fn session_exposes_element_access_key_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><div id='box' accesskey='x'></div><div id='out'></div><script>const box = document.getElementById('box'); const before = box.accessKey; box.accessKey = 'y'; document.getElementById('out').textContent = before + ':' + box.accessKey + ':' + box.getAttribute('accesskey');</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("element.accessKey should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "x:y:y");
}

#[test]
fn session_exposes_element_slot_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><div id='box' slot='start'></div><div id='out'></div><script>const box = document.getElementById('box'); const before = box.slot; box.slot = 'end'; document.getElementById('out').textContent = before + ':' + box.slot + ':' + box.getAttribute('slot');</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("element.slot should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "start:end:end");
}

#[test]
fn session_exposes_element_autocapitalize_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><div id='box' autocapitalize='sentences'></div><div id='out'></div><script>const box = document.getElementById('box'); const before = box.autocapitalize; box.autocapitalize = 'words'; document.getElementById('out').textContent = before + ':' + box.autocapitalize + ':' + box.getAttribute('autocapitalize');</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("element.autocapitalize should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "sentences:words:words"
    );
}

#[test]
fn session_exposes_element_spellcheck_and_input_mode_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><div id='out'></div><div id='parent' spellcheck='false'><span id='child'>Edit</span></div><div id='mode' inputmode='text'></div><script>const child = document.getElementById('child'); const beforeSpell = String(child.spellcheck); child.spellcheck = true; const afterSpell = String(child.spellcheck) + ':' + child.getAttribute('spellcheck'); const mode = document.getElementById('mode'); const beforeMode = mode.inputMode; mode.inputMode = 'numeric'; document.getElementById('out').textContent = beforeSpell + ':' + afterSpell + ':' + beforeMode + ':' + mode.inputMode + ':' + mode.getAttribute('inputmode');</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("element.spellcheck and element.inputMode should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "false:true:true:text:numeric:numeric"
    );
}

#[test]
fn session_rejects_non_element_spellcheck_and_input_mode_access_regression() {
    Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some("<main id='root'></main><script>document.createTextNode('x').spellcheck = false;</script>".to_string()),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-element spellcheck access should fail explicitly");

    Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some("<main id='root'></main><script>document.createTextNode('x').inputMode = 'numeric';</script>".to_string()),
        local_storage: BTreeMap::new(),
    })
    .expect_err("non-element inputMode access should fail explicitly");
}

#[test]
fn session_exposes_element_hidden_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><div id='box' hidden></div><div id='out'></div><script>const box = document.getElementById('box'); const before = String(box.hidden); box.hidden = false; const afterClear = String(box.hidden) + ':' + String(box.hasAttribute('hidden')); box.hidden = true; document.getElementById('out').textContent = before + ':' + afterClear + ':' + String(box.hidden) + ':' + String(box.hasAttribute('hidden'));</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("element.hidden should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "true:false:false:true:true"
    );
}

#[test]
fn session_exposes_element_dir_and_lang_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><div id='box'></div><div id='out'></div><script>const box = document.getElementById('box'); const before = String(box.dir) + '|' + String(box.lang); box.dir = 'rtl'; box.lang = 'fr'; const afterSet = String(box.dir) + '|' + String(box.lang) + '|' + String(box.getAttribute('dir')) + '|' + String(box.getAttribute('lang')); document.getElementById('out').textContent = before + ';' + afterSet;</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("element dir and lang should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "|;rtl|fr|rtl|fr"
    );
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
fn session_resolves_document_images_iterator_helpers_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><img id='hero' name='hero' alt='Hero'><img id='thumb' name='thumb' alt='Thumb'></div><div id='out'></div><script>const images = document.images; const keys = images.keys(); const values = images.values(); const entries = images.entries(); const firstKey = keys.next(); const firstValue = values.next(); const firstEntry = entries.next(); let out = ''; images.forEach((element, index, list) => { out += String(index) + ':' + element.getAttribute('id') + ':' + String(list.length) + ';'; }); document.getElementById('out').textContent = String(firstKey.value) + ':' + firstValue.value.getAttribute('id') + ':' + String(firstEntry.value.index) + ':' + firstEntry.value.value.getAttribute('id') + ':' + out;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("document.images iterator helpers should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "0:hero:0:hero:0:hero:2;1:thumb:2;"
    );
}

#[test]
fn session_resolves_document_links_iterator_helpers_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><img id='hero' name='hero' alt='Hero'><img name='thumb' alt='Thumb'><a id='docs' href='/docs'>Docs</a><a id='plain'>Plain</a><area id='map' name='map' href='/map'></div><div id='out'></div><script>const links = document.links; const keys = links.keys(); const values = links.values(); const entries = links.entries(); const firstKey = keys.next(); const firstValue = values.next(); const firstEntry = entries.next(); let out = ''; links.forEach((element, index, list) => { out += String(index) + ':' + element.getAttribute('id') + ':' + String(list.length) + ';'; }); document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(firstKey.value) + ':' + firstValue.value.getAttribute('id') + ':' + String(firstEntry.value.index) + ':' + firstEntry.value.value.getAttribute('id') + ':' + out;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("document.links iterator helpers should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "0:docs:0:docs:0:docs:2;1:map:2;"
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
fn session_resolves_document_plugins_iterator_helpers_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><embed id='first-embed'><embed name='second-embed'></div><div id='out'></div><script>const plugins = document.plugins; const keys = plugins.keys(); const values = plugins.values(); const entries = plugins.entries(); const firstKey = keys.next(); const firstValue = values.next(); const firstEntry = entries.next(); document.getElementById('out').textContent = String(firstKey.value) + ':' + firstValue.value.getAttribute('id') + ':' + String(firstEntry.value.index) + ':' + firstEntry.value.value.getAttribute('id');</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("document.plugins iterator helpers should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "0:first-embed:0:first-embed"
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
fn session_resolves_collection_to_string_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>const childNodes = document.childNodes; document.getElementById('out').textContent = childNodes.toString() + ':' + document.plugins.toString() + ':' + window.navigator.mimeTypes.toString();</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("collection toString helpers should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "[object NodeList]:[object HTMLCollection]:[object MimeTypeArray]"
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
            "<main id='root' class='primary'></main><script>document.getElementById('root').matches('main:not([data-kind=primary x y])');</script>"
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
            "<main id='root'><section id='section'><input id='field'></section><div id='outside'>outside</div></main><div id='out'></div><script>document.getElementById('field').addEventListener('focus', () => { const field = document.querySelector(':focus'); const focusVisible = document.querySelector(':focus-visible'); const section = document.getElementById('section'); const root = document.getElementById('root'); document.getElementById('out').textContent = field.getAttribute('id') + ':' + focusVisible.getAttribute('id') + ':' + String(section.matches(':focus-visible')) + ':' + String(root.matches(':focus-visible')) + ':' + String(section.matches(':focus-within')) + ':' + String(root.matches(':focus-within')); });</script>"
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
        session.dom().select(":focus-visible").unwrap(),
        vec![field_id]
    );
    assert_eq!(
        session.dom().select("#field:focus").unwrap(),
        vec![field_id]
    );
    assert_eq!(
        session.dom().select("#field:focus-visible").unwrap(),
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
        "field:field:false:false:true:true"
    );

    session.blur_node(field_id).expect("blur should work");
    assert!(session.dom().select(":focus").unwrap().is_empty());
    assert!(session.dom().select(":focus-visible").unwrap().is_empty());
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
fn session_resolves_element_click_focus_and_blur_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<input id='agree' type='checkbox'><input id='first'><div id='out'></div><script>const checkbox = document.getElementById('agree'); const first = document.getElementById('first'); checkbox.click(); first.focus(); const focused = document.activeElement.getAttribute('id'); first.blur(); document.getElementById('out').textContent = String(checkbox.checked) + ':' + String(focused);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("Element.click/focus/blur should be wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(session.dom().text_content_for_node(out_id), "true:first");
}

#[test]
fn session_resolves_window_navigator_on_line_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='out'></div><script>document.getElementById('out').textContent = String(window.navigator.onLine) + ':' + String(window.navigator.webdriver) + ':' + String(window.navigator.appCodeName) + ':' + String(window.navigator.appName) + ':' + String(window.navigator.appVersion) + ':' + String(window.navigator.product) + ':' + String(window.navigator.productSub) + ':' + String(window.navigator.vendor) + ':' + String(window.navigator.vendorSub) + ':' + String(window.navigator.pdfViewerEnabled) + ':' + String(window.navigator.doNotTrack) + ':' + String(window.navigator.javaEnabled()) + ':' + String(window.navigator.hardwareConcurrency) + ':' + String(window.navigator.maxTouchPoints) + ':' + window.navigator.userLanguage + ':' + window.navigator.browserLanguage + ':' + window.navigator.systemLanguage + ':' + window.navigator.oscpu + ':' + String(window.navigator.languages.length) + ':' + window.navigator.languages.item(0) + ':' + window.navigator.languages.toString();</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("navigator.onLine should be wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "true:false:browser-tester-next:browser-tester-next:browser-tester-next:browser-tester-next:browser-tester-next:browser-tester-next:browser-tester-next:false:unspecified:false:8:0:en-US:en-US:en-US:unknown:1:en-US:[object DOMStringList]"
    );
}

#[test]
fn session_rejects_window_navigator_languages_assignment_regression() {
    let mut runtime = ScriptRuntime::new();
    let mut host = bt_runtime::Session::new(bt_runtime::SessionConfig::default())
        .expect("session should build");

    let error = runtime
        .eval_program(
            "window.navigator.languages = null;",
            "inline-script",
            &mut host,
        )
        .expect_err("navigator.languages should be read-only");

    assert!(
        error
            .to_string()
            .contains("cannot assign to `languages` on navigator value")
    );
}

#[test]
fn session_resolves_window_navigator_plugins_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><embed id='first-embed'><embed name='second-embed'></div><div id='out'></div><script>document.getElementById('out').textContent = String(window.navigator.plugins.length) + ':' + String(window.navigator.plugins.namedItem('first-embed')) + ':' + String(window.navigator.plugins.namedItem('missing'));</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("navigator.plugins should be wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "2:[object Element]:null"
    );
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
fn session_resolves_document_all_iterator_helpers_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><span id='first'>First</span><span id='second'>Second</span></div><div id='out'></div><script id='script'>const all = document.all; const keys = all.keys(); const values = all.values(); const entries = all.entries(); const firstKey = keys.next(); const firstValue = values.next(); const firstEntry = entries.next(); let out = ''; all.forEach((element, index, list) => { out += String(index) + ':' + element.getAttribute('id') + ':' + String(list.length) + ';'; }); document.getElementById('out').textContent = String(firstKey.value) + ':' + firstValue.value.getAttribute('id') + ':' + String(firstEntry.value.index) + ':' + firstEntry.value.value.getAttribute('id') + ':' + out;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("document.all iterator helpers should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "0:root:0:root:0:root:5;1:first:5;2:second:5;3:out:5;4:script:5;"
    );
}

#[test]
fn session_rejects_html_collection_reserved_named_property_access_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><span id='first'>First</span></main><script>document.getElementById('root').children.item;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("reserved HTMLCollection named property access should fail explicitly");

    assert!(error.to_string().contains("Script error"));
    assert!(error.to_string().contains("unsupported member access"));
    assert!(error.to_string().contains("`item`"));
    assert!(error.to_string().contains("html collection value"));
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
fn session_resolves_select_options_iterator_helpers_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><select id='mode'><option id='first' value='a'>A</option><option id='second' value='b'>B</option></select></div><div id='out'></div><script>const options = document.getElementById('mode').options; const keys = options.keys(); const values = options.values(); const entries = options.entries(); const firstKey = keys.next(); const firstValue = values.next(); const firstEntry = entries.next(); let out = ''; options.forEach((element, index, list) => { out += String(index) + ':' + element.getAttribute('id') + ':' + String(list.length) + ';'; }); document.getElementById('out').textContent = String(firstKey.value) + ':' + firstValue.value.getAttribute('id') + ':' + String(firstEntry.value.index) + ':' + firstEntry.value.value.getAttribute('id') + ':' + out;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("select.options iterator helpers should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "0:first:0:first:0:first:2;1:second:2;"
    );
}

#[test]
fn session_resolves_select_selected_options_iterator_helpers_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<div id='root'><select id='mode'><option id='first' value='a' selected>A</option><option id='second' value='b' selected>B</option></select></div><div id='out'></div><script>const selected = document.getElementById('mode').selectedOptions; const keys = selected.keys(); const values = selected.values(); const entries = selected.entries(); const firstKey = keys.next(); const firstValue = values.next(); const firstEntry = entries.next(); let out = ''; selected.forEach((element, index, list) => { out += String(index) + ':' + element.getAttribute('id') + ':' + String(list.length) + ';'; }); document.getElementById('out').textContent = String(firstKey.value) + ':' + firstValue.value.getAttribute('id') + ':' + String(firstEntry.value.index) + ':' + firstEntry.value.value.getAttribute('id') + ':' + out;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("select.selectedOptions iterator helpers should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "0:first:0:first:0:first:2;1:second:2;"
    );
}

#[test]
fn session_resolves_table_rows_and_row_cells_iterator_helpers_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<table id='table'><thead id='head'><tr id='head-row'><th id='head-cell'>H</th></tr></thead><tbody id='body'><tr id='first-row'><td id='first-cell'>A</td></tr></tbody><tfoot id='foot'><tr id='foot-row'><td id='foot-cell'>F</td></tr></tfoot></table><div id='out'></div><script>const table = document.getElementById('table'); const row = document.getElementById('first-row'); const tableKeys = table.rows.keys(); const tableValues = table.rows.values(); const tableEntries = table.rows.entries(); const rowKeys = row.cells.keys(); const rowValues = row.cells.values(); const rowEntries = row.cells.entries(); const firstTableKey = tableKeys.next(); const firstTableValue = tableValues.next(); const firstTableEntry = tableEntries.next(); const firstRowKey = rowKeys.next(); const firstRowValue = rowValues.next(); const firstRowEntry = rowEntries.next(); let out = ''; table.rows.forEach((element, index, list) => { out += 'T' + String(index) + ':' + element.getAttribute('id') + ':' + String(list.length) + ';'; }); row.cells.forEach((element, index, list) => { out += 'R' + String(index) + ':' + element.getAttribute('id') + ':' + String(list.length) + ';'; }); document.getElementById('out').textContent = String(firstTableKey.value) + ':' + firstTableValue.value.getAttribute('id') + ':' + String(firstTableEntry.value.index) + ':' + firstTableEntry.value.value.getAttribute('id') + ':' + String(firstRowKey.value) + ':' + firstRowValue.value.getAttribute('id') + ':' + String(firstRowEntry.value.index) + ':' + firstRowEntry.value.value.getAttribute('id') + ':' + out;</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("table.rows and row.cells iterator helpers should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "0:head-row:0:head-row:0:first-cell:0:first-cell:T0:head-row:3;T1:first-row:3;T2:foot-row:3;R0:first-cell:1;"
    );
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
fn session_compare_document_position_reports_tree_order_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<html><body><main id='root'><span id='a'><em id='c'>Child</em></span><span id='b'>Sibling</span></main><div id='out'></div><script>const a = document.getElementById('a'); const b = document.getElementById('b'); const c = document.getElementById('c'); const detached = document.createElement('p'); document.getElementById('out').textContent = String(document.compareDocumentPosition(a)) + ':' + String(a.compareDocumentPosition(document)) + ':' + String(a.compareDocumentPosition(b)) + ':' + String(b.compareDocumentPosition(a)) + ':' + String(a.compareDocumentPosition(c)) + ':' + String(c.compareDocumentPosition(a)) + ':' + String(a.compareDocumentPosition(detached)) + ':' + String(detached.compareDocumentPosition(a)) + ':' + String(document.compareDocumentPosition(document));</script></body></html>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("compareDocumentPosition should remain deterministic");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "20:10:4:2:20:10:37:35:0"
    );
}

#[test]
fn session_rejects_node_before_self_insertion_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'></main><script>const root = document.getElementById('root'); const child = document.createTextNode('Hello'); root.appendChild(child); child.before(child);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("self-insertion through Node.before should fail explicitly");

    assert!(
        error
            .to_string()
            .contains("a node cannot be inserted relative to itself")
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
fn session_serializes_insert_adjacent_element_and_text_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><section id='target'><button id='old' class='primary'>Old</button></section></main><div id='out'></div><script>const target = document.getElementById('target'); const before = target.insertAdjacentElement('beforebegin', document.createElement('aside')); before.setAttribute('id', 'before'); before.textContent = 'Before'; target.insertAdjacentText('afterbegin', 'First'); const last = target.insertAdjacentElement('beforeend', document.createElement('span')); last.setAttribute('id', 'last'); last.textContent = 'Last'; const after = target.insertAdjacentElement('afterend', document.createElement('aside')); after.setAttribute('id', 'after'); after.textContent = 'After'; target.insertAdjacentText('beforeend', 'Tail'); document.getElementById('out').textContent = document.getElementById('root').innerHTML + '|' + target.innerHTML + '|' + String(before) + ':' + String(after);</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("insertAdjacentElement and insertAdjacentText should remain wired through Session");

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
fn session_rejects_invalid_insert_adjacent_element_position_regression() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><section id='target'></section><div id='out'></div><script>document.getElementById('target').insertAdjacentElement('middle', document.createElement('aside'));</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("invalid insertAdjacentElement positions should fail explicitly");
    assert!(
        error
            .to_string()
            .contains("unsupported insertAdjacentElement position")
    );
}

#[test]
fn session_rejects_void_insert_adjacent_text_regression() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><img id='image'></main><script>document.getElementById('image').insertAdjacentText('beforeend', 'Bad');</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("void elements should reject insertAdjacentText explicitly");

    assert!(
        error
            .to_string()
            .contains("insertAdjacentText is not supported on void elements")
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
fn session_writes_document_html_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<html><body><main id='root'><div id='out'></div><script>document.write('<span id=\"written\">Written</span>'); document.getElementById('out').textContent = document.body.lastElementChild.getAttribute('id') + ':' + document.getElementById('written').textContent;</script></main></body></html>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("document.write should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "written:Written"
    );
    assert_eq!(session.dom().select("#written").unwrap().len(), 1);
}

#[test]
fn session_writes_document_html_with_newline_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<html><body><main id='root'><div id='out'></div><script>document.writeln('<span id=\"written\">Written</span>'); document.getElementById('out').textContent = document.body.lastElementChild.getAttribute('id') + ':' + String(document.body.lastChild.nodeType) + ':' + document.getElementById('written').textContent;</script></main></body></html>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("document.writeln should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "written:3:Written"
    );
    assert_eq!(session.dom().select("#written").unwrap().len(), 1);
}

#[test]
fn session_opens_document_and_writes_fresh_html_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<html><head><title>Title</title></head><body><main id='root'><div id='out'></div></main><script>const opened = document.open(); document.write('<span id=\"written\">Written</span><div id=\"out\"></div>'); const closed = document.close(); document.getElementById('out').textContent = String(opened) + ':' + String(closed) + ':' + String(document.documentElement) + ':' + String(document.body);</script></body></html>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("document.open should remain wired through Session");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "[object Document]:[object Document]:[object Element]:null"
    );
    assert_eq!(session.dom().select("#written").unwrap().len(), 1);
    assert!(session.dom().select("#root").unwrap().is_empty());
}

#[test]
fn session_serializes_mixed_quote_attribute_values_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><div id='target'></div><div id='out'></div><script>const target = document.getElementById('target'); target.setAttribute('data-label', \"a'b\\\"c\"); document.getElementById('out').textContent = String(target.outerHTML);</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("mixed-quote serialization should succeed explicitly");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "<div data-label=\"a'b&quot;c\" id=\"target\"></div>"
    );
}

#[test]
fn session_serializes_mixed_quote_attribute_values_through_document_write_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><div id='out'></div><script>document.write('<div id=\"target\" data-label=\"a\\'b&quot;c\"></div>'); document.getElementById('out').textContent = document.getElementById('target').outerHTML;</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("document.write mixed-quote serialization should succeed");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "<div data-label=\"a'b&quot;c\" id=\"target\"></div>"
    );
}

#[test]
fn session_decodes_common_named_character_entities_through_document_write_regression() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><div id='out'></div><script>document.write('<div id=\"target\" data-label=\"a&nbsp;b\">A&nbsp;B</div>'); const target = document.getElementById('target'); document.getElementById('out').textContent = target.getAttribute('data-label') + ':' + target.textContent + ':' + target.outerHTML;</script></main>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("document.write common named entity decoding should succeed");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "a\u{a0}b:A\u{a0}B:<div data-label=\"a\u{a0}b\" id=\"target\">A\u{a0}B</div>"
    );
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
fn session_rejects_storage_length_assignment_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='out'></main><script>window.localStorage.length = '2';</script>".to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("storage length should be read-only");

    assert!(error.to_string().contains("Script error"));
    assert!(
        error
            .to_string()
            .contains("cannot assign to `length` on storage value")
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

#[test]
fn session_supports_namespaced_attribute_nodes_explicitly() {
    let session = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root'><div id='box'></div><div id='out'></div><script>const box = document.getElementById('box'); const attrs = box.attributes; const created = document.createAttributeNS('urn:test', 'svg:stroke'); created.value = 'azure'; const before = attrs.length; const previous = attrs.setNamedItemNS(created); const during = attrs.length; const snapshot = box.getAttributeNodeNS('urn:test', 'stroke'); const snapshotOwner = snapshot.ownerElement; const createdOwner = created.ownerElement; const removed = attrs.removeNamedItemNS('urn:test', 'stroke'); const removedOwner = removed.ownerElement; document.getElementById('out').textContent = String(before) + ':' + String(previous) + ':' + String(during) + ':' + String(snapshot) + ':' + snapshot.name + ':' + String(snapshot.namespaceURI) + ':' + snapshot.localName + ':' + String(snapshot.prefix) + ':' + snapshot.value + ':' + String(snapshotOwner) + ':' + String(createdOwner) + ':' + String(removed) + ':' + String(removedOwner) + ':' + String(box.getAttributeNodeNS('urn:test', 'stroke'));</script></main>".to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect("session should build");

    let out_id = session.dom().select("#out").unwrap()[0];
    assert_eq!(
        session.dom().text_content_for_node(out_id),
        "1:null:2:[object Attr]:svg:stroke:urn:test:stroke:svg:azure:[object Element]:[object Element]:[object Attr]:null:null"
    );
}
