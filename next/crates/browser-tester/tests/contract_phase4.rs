use browser_tester_next::Harness;

#[test]
fn click_events_bubble_beyond_the_target_phase() -> browser_tester_next::Result<()> {
    let mut harness = Harness::from_html(
        "<div id='parent'><div id='child'></div></div><div id='out'></div><script>document.getElementById('child').addEventListener('click', () => { document.getElementById('out').textContent = 'target'; }); document.getElementById('parent').addEventListener('click', () => { document.getElementById('out').textContent += ':parent'; }); document.addEventListener('click', () => { document.getElementById('out').textContent += ':document'; }); window.addEventListener('click', () => { document.getElementById('out').textContent += ':window'; });</script>",
    )?;

    harness.click("#child")?;
    harness.assert_text("#out", "target:parent:document:window")?;
    Ok(())
}

#[test]
fn prevent_default_cancels_click_default_action() -> browser_tester_next::Result<()> {
    let mut harness = Harness::from_html(
        "<input id='agree' type='checkbox'><div id='out'></div><script>document.getElementById('agree').addEventListener('click', (event) => { event.preventDefault(); }); document.getElementById('agree').addEventListener('change', () => { document.getElementById('out').textContent = String(document.getElementById('agree').checked); });</script>",
    )?;

    harness.click("#agree")?;
    harness.assert_checked("#agree", false)?;
    harness.assert_text("#out", "")?;
    Ok(())
}

#[test]
fn focus_and_blur_are_publicly_supported() -> browser_tester_next::Result<()> {
    let mut harness = Harness::from_html(
        "<input id='first'><input id='second'><div id='out'></div><script>document.getElementById('first').addEventListener('blur', () => { document.getElementById('second').textContent = 'after-blur'; }); document.getElementById('second').addEventListener('focus', () => { document.getElementById('out').textContent = document.getElementById('second').textContent; });</script>",
    )?;

    harness.focus("#first")?;
    harness.focus("#second")?;
    harness.assert_text("#out", "after-blur")?;
    Ok(())
}

#[test]
fn set_select_value_updates_selection_and_fires_change() -> browser_tester_next::Result<()> {
    let mut harness = Harness::from_html(
        "<select id='mode'><option value='a'>A</option><option value='b'>B</option></select><div id='out'></div><script>document.getElementById('mode').addEventListener('change', () => { document.getElementById('out').textContent = document.getElementById('mode').value; });</script>",
    )?;

    harness.set_select_value("#mode", "b")?;
    harness.assert_value("#mode", "b")?;
    harness.assert_text("#out", "b")?;
    Ok(())
}

#[test]
fn fetch_uses_mock_response_and_records_calls() -> browser_tester_next::Result<()> {
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
fn missing_fetch_mock_returns_a_mock_error() -> browser_tester_next::Result<()> {
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
fn dialogs_clipboard_and_location_are_wired() -> browser_tester_next::Result<()> {
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
fn dialogs_and_clipboard_reads_require_seeded_values() -> browser_tester_next::Result<()> {
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
fn file_input_selection_updates_dom_and_capture() -> browser_tester_next::Result<()> {
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
fn set_files_rejects_non_file_inputs() -> browser_tester_next::Result<()> {
    let mut harness = Harness::from_html("<input id='name'>")?;

    let error = harness
        .set_files("#name", ["report.csv"])
        .expect_err("set_files should reject non-file inputs");
    assert!(error.to_string().contains("file input control"));
    Ok(())
}
