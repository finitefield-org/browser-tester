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
    });

    assert_eq!(session.config().url, "https://example.test/app");
    assert_eq!(session.dom().source_html(), Some("<div id='app'></div>"));
    assert_eq!(
        session
            .mocks()
            .storage()
            .local()
            .get("theme")
            .map(String::as_str),
        Some("light")
    );
}
