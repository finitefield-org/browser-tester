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
