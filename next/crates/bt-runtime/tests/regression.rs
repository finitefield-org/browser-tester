use std::collections::BTreeMap;

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
    assert!(registry.downloads().artifacts().is_empty());
    assert!(registry.file_input().selections().is_empty());
    assert!(registry.storage().local().is_empty());
    assert!(registry.storage().session().is_empty());
}

#[test]
fn session_rejects_unsupported_selector_syntax_in_closest_explicitly() {
    let error = Session::new(SessionConfig {
        url: "https://example.test/app".to_string(),
        html: Some(
            "<main id='root' class='primary'></main><script>document.getElementById('root').closest('main:nth-child(2)');</script>"
                .to_string(),
        ),
        local_storage: BTreeMap::new(),
    })
    .expect_err("unsupported selector syntax should fail explicitly");

    assert!(error.to_string().contains("Script error"));
    assert!(error.to_string().contains("supported forms are #id, .class, tag, tag.class, #id.class, [attr], descendant combinators like `A B`, adjacent sibling combinators like `A + B`, general sibling combinators like `A ~ B`, and child combinators like `A > B`"));
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
