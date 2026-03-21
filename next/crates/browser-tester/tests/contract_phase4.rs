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
fn document_location_is_wired_through_script_and_location_mock() -> browser_tester_next::Result<()>
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
fn document_url_assignment_is_rejected() -> browser_tester_next::Result<()> {
    let error = Harness::from_html(
        "<main id='out'></main><script>document.URL = 'https://example.test/next';</script>",
    )
    .expect_err("document.URL should be read-only");

    assert!(error
        .to_string()
        .contains("unsupported assignment target"));
    assert!(error.to_string().contains("URL"));
    Ok(())
}

#[test]
fn document_base_uri_aliases_are_publicly_supported() -> browser_tester_next::Result<()> {
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
fn document_origin_aliases_are_publicly_supported() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><span id='child'></span></main><div id='out'></div><script>const root = document.getElementById('root'); const child = document.getElementById('child'); document.getElementById('out').textContent = document.origin + ':' + window.origin + ':' + root.origin + ':' + child.origin;</script>",
    )?;

    harness.assert_text(
        "#out",
        "https://app.local:https://app.local:https://app.local:https://app.local",
    )?;
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
fn download_capture_is_publicly_wired() -> browser_tester_next::Result<()> {
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
fn capture_download_rejects_blank_file_names() -> browser_tester_next::Result<()> {
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
