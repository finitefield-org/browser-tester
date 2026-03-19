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
    assert_eq!(harness.debug().dom_node_count(), 2);

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
fn from_html_uses_default_url_and_records_source_html() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html("<main id='app'></main>")?;

    assert_eq!(harness.debug().url(), "https://app.local/");
    assert_eq!(
        harness.debug().source_html(),
        Some("<main id='app'></main>")
    );
    assert_eq!(harness.debug().dom_node_count(), 2);
    Ok(())
}

#[test]
fn advance_time_rejects_negative_deltas() {
    let mut harness = Harness::builder().build().expect("builder should succeed");

    let error = harness
        .advance_time(-1)
        .expect_err("negative time deltas should fail");

    assert_eq!(
        error.to_string(),
        "Timer error: advance_time requires a non-negative delta"
    );
    assert_eq!(harness.now_ms(), 0);
}

#[test]
fn storage_seed_registry_is_available_through_the_mock_view() {
    let mut harness = Harness::builder().build().expect("builder should succeed");

    {
        let mut mocks = harness.mocks_mut();
        let storage = mocks.storage();
        storage.seed_local("token", "abc");
        storage.seed_session("session-token", "xyz");

        assert_eq!(
            storage.local().get("token").map(String::as_str),
            Some("abc")
        );
        assert_eq!(
            storage.session().get("session-token").map(String::as_str),
            Some("xyz")
        );
    }

    harness.mocks_mut().reset_all();
    assert!(harness.mocks_mut().storage().local().is_empty());
    assert!(harness.mocks_mut().storage().session().is_empty());
}
