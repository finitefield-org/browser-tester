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
