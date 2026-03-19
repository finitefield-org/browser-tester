use browser_tester_next::Harness;

#[test]
fn builder_creates_empty_session_without_html() -> browser_tester_next::Result<()> {
    let harness = Harness::builder().build()?;
    assert_eq!(harness.debug().url(), "https://app.local/");
    assert_eq!(harness.debug().source_html(), None);
    assert_eq!(harness.debug().dom_node_count(), 1);
    Ok(())
}

#[test]
fn builder_keeps_url_html_and_local_storage_configuration() -> browser_tester_next::Result<()> {
    let mut harness = Harness::builder()
        .url("https://example.test/app")
        .html("<main id='app'></main>")
        .local_storage([("token", "abc"), ("theme", "light")])
        .build()?;

    assert_eq!(harness.debug().url(), "https://example.test/app");
    assert_eq!(
        harness.debug().source_html(),
        Some("<main id='app'></main>")
    );
    assert_eq!(
        harness
            .debug()
            .local_storage()
            .get("token")
            .map(String::as_str),
        Some("abc")
    );

    harness.advance_time(25)?;
    assert_eq!(harness.now_ms(), 25);

    harness
        .mocks_mut()
        .fetch()
        .respond_text("https://app.local/api/message", 200, "ok");
    assert_eq!(harness.mocks_mut().fetch().responses().len(), 1);
    Ok(())
}

#[test]
fn phase_zero_actions_fail_explicitly_until_implemented() {
    let mut harness = Harness::builder().build().expect("builder should succeed");
    let error = harness.click("#submit").expect_err("click should be gated");
    assert!(
        error
            .to_string()
            .contains("Phase 3 after selector and event support land")
    );
}
