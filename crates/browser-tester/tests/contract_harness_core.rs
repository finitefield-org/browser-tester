use browser_tester::{Error, FileInputFile, Harness, Result};

#[test]
fn stable_core_actions_and_assertions_work_together() -> Result<()> {
    let html = "<button id='run' type='button'>run</button><input id='name'><input id='agree' type='checkbox'><p id='clicked'></p><script>document.getElementById('run').addEventListener('click', () => { document.getElementById('clicked').textContent = 'clicked'; });</script>";

    let mut h = Harness::from_html(html)?;
    h.type_text("#name", "Alice")?;
    h.set_checked("#agree", true)?;
    h.click("#run")?;

    h.assert_text("#clicked", "clicked")?;
    h.assert_value("#name", "Alice")?;
    h.assert_checked("#agree", true)?;
    Ok(())
}

#[test]
fn stable_core_assert_exists_reports_presence_and_missing_selectors() -> Result<()> {
    let h = Harness::from_html("<p id='present'>ok</p>")?;

    h.assert_exists("#present")?;

    let err = h
        .assert_exists("#missing")
        .expect_err("missing selector should fail");
    assert!(matches!(err, Error::Assertion(_)));
    assert!(err.to_string().contains("expected selector `#missing`"));

    Ok(())
}

#[test]
fn stable_core_constructors_and_time_controls_work() -> Result<()> {
    let mut h = Harness::from_html_with_url_and_local_storage(
        "https://app.local/start",
        "<p id='out'></p>",
        [("token", "seed"), ("mode", "debug")],
    )?;

    assert_eq!(h.debug().url(), "https://app.local/start");
    assert_eq!(
        h.debug().local_storage().get("token").map(String::as_str),
        Some("seed")
    );
    assert_eq!(
        h.debug().local_storage().get("mode").map(String::as_str),
        Some("debug")
    );
    assert_eq!(h.now_ms(), 0);
    h.advance_time(25)?;
    assert_eq!(h.now_ms(), 25);
    Ok(())
}

#[test]
fn stable_test_mock_fetch_contract_is_direct() -> Result<()> {
    let mut h = Harness::builder().build()?;
    h.mocks_mut()
        .fetch()
        .respond_text("https://app.local/api/message", 200, "hello");

    let response = h.fetch("https://app.local/api/message")?;
    assert_eq!(response.url, "https://app.local/api/message");
    assert_eq!(response.status, 200);
    assert_eq!(response.body, "hello");
    assert_eq!(h.mocks_mut().fetch().calls().len(), 1);
    Ok(())
}

#[test]
fn stable_test_mock_clipboard_contract_is_direct() -> Result<()> {
    let mut h = Harness::from_html("<p id='out'></p>")?;
    h.mocks_mut().clipboard().seed_text("seeded");

    assert_eq!(h.read_clipboard()?, "seeded");
    h.write_clipboard("copied")?;
    assert_eq!(h.read_clipboard()?, "copied");
    assert_eq!(h.mocks_mut().clipboard().writes(), &["copied".to_string()]);
    Ok(())
}

#[test]
fn stable_test_mock_clipboard_error_controls_are_direct() -> Result<()> {
    let html = r#"
      <button id='run'>run</button>
      <p id='out'></p>
      <script>
        document.getElementById('run').addEventListener('click', () => {
          try {
            window.navigator.clipboard.writeText('saved');
            document.getElementById('out').textContent =
              'ok:' + window.navigator.clipboard.readText();
          } catch (reason) {
            document.getElementById('out').textContent =
              'err:' + String(reason && reason.message ? reason.message : reason);
          }
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.set_clipboard_write_error(Some("WriteBlocked"));
    h.click("#run")?;
    h.assert_text("#out", "err:Mock error: WriteBlocked")?;

    h.clear_clipboard_errors();
    h.click("#run")?;
    h.assert_text("#out", "ok:saved")?;
    assert_eq!(h.clipboard_text(), "saved");
    Ok(())
}

#[test]
fn stable_test_mock_location_contract_is_direct() -> Result<()> {
    let mut h = Harness::from_html_with_url(
        "https://app.local/start",
        "<a id='go' href='https://app.local/next'>next</a>",
    )?;

    h.navigate("https://app.local/next")?;

    assert_eq!(h.debug().url(), "https://app.local/next");
    assert_eq!(
        h.mocks_mut().location().current_url(),
        Some("https://app.local/next")
    );
    assert_eq!(
        h.mocks_mut().location().navigations(),
        &["https://app.local/next".to_string()]
    );
    Ok(())
}

#[test]
fn stable_test_mock_file_input_contract_is_direct() -> Result<()> {
    let mut h = Harness::from_html("<input id='upload' type='file' multiple><p id='out'></p>")?;
    h.set_files(
        "#upload",
        [
            FileInputFile::from_text("first.txt", "one").with_mime_type("text/plain"),
            FileInputFile::from_text("second.txt", "two").with_mime_type("text/plain"),
        ],
    )?;

    h.assert_value("#upload", "first.txt, second.txt")?;
    assert_eq!(
        h.mocks_mut().file_input().selections()[0].files,
        vec![
            FileInputFile::from_text("first.txt", "one").with_mime_type("text/plain"),
            FileInputFile::from_text("second.txt", "two").with_mime_type("text/plain"),
        ]
    );
    Ok(())
}

#[test]
fn stable_core_scheduler_controls_are_direct() -> Result<()> {
    let html = "<p id='out'></p>";

    let mut h = Harness::from_html(html)?;
    assert_eq!(h.now_ms(), 0);
    h.advance_time(0)?;
    h.flush()?;
    h.assert_text("#out", "")?;
    Ok(())
}

#[test]
fn stable_core_trace_and_determinism_controls_are_direct() -> Result<()> {
    let html = r#"
      <button id='run'>run</button>
      <p id='out'></p>
      <script>
        document.getElementById('run').addEventListener('click', () => {
          document.getElementById('out').textContent =
            Math.random() + ':' + Math.random();
          setTimeout(() => {}, 0);
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.set_random_seed(7);
    h.click("#run")?;
    let first = h.dump_dom("#out")?;

    h.set_random_seed(7);
    h.click("#run")?;
    let second = h.dump_dom("#out")?;
    assert_eq!(first, second);

    h.enable_trace(true);
    h.set_trace_stderr(false);
    h.set_trace_timers(false);
    h.click("#run")?;
    let logs = h.take_trace_logs();
    assert!(logs.iter().any(|line| line.contains("[event] click")));
    assert!(logs.iter().all(|line| !line.contains("[timer]")));

    h.set_trace_events(false);
    h.set_trace_timers(true);
    h.click("#run")?;
    let logs = h.take_trace_logs();
    assert!(
        logs.iter()
            .any(|line| line.contains("[timer] schedule timeout"))
    );
    assert!(logs.iter().all(|line| !line.contains("[event]")));

    h.set_trace_events(true);
    h.set_trace_log_limit(2)?;
    h.dispatch("#run", "alpha")?;
    h.dispatch("#run", "beta")?;
    h.dispatch("#run", "gamma")?;
    let logs = h.take_trace_logs();
    assert_eq!(logs.len(), 2);
    assert!(logs.iter().any(|line| line.contains("done beta")));
    assert!(logs.iter().any(|line| line.contains("done gamma")));
    Ok(())
}

#[test]
fn stable_core_limit_validation_errors_are_direct() -> Result<()> {
    let mut h = Harness::from_html("<p id='out'>ok</p>")?;

    let trace_limit_err = h
        .set_trace_log_limit(0)
        .expect_err("zero trace log limit should be rejected");
    match trace_limit_err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("set_trace_log_limit requires at least 1 entry"));
        }
        other => panic!("unexpected error: {other:?}"),
    }

    let timer_limit_err = h
        .set_timer_step_limit(0)
        .expect_err("zero timer step limit should be rejected");
    match timer_limit_err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("set_timer_step_limit requires at least 1 step"));
        }
        other => panic!("unexpected error: {other:?}"),
    }

    Ok(())
}

#[test]
fn stable_test_mock_dialog_and_match_media_controls_are_direct() -> Result<()> {
    let html = "<p id='out'></p><script>const media = window.matchMedia('(min-width: 768px)'); document.getElementById('out').textContent = String(media.matches) + ':' + media.media;</script>";

    let mut h = Harness::builder()
        .html(html)
        .match_media([("(min-width: 768px)", true)])
        .build()?;
    h.mocks_mut().dialogs().push_confirm(true);
    h.mocks_mut().dialogs().push_prompt(Some("kazu"));

    h.assert_text("#out", "true:(min-width: 768px)")?;
    assert!(h.confirm("continue?")?);
    assert_eq!(h.prompt("name?")?, Some("kazu".to_string()));
    h.alert("hello kazu")?;
    h.print()?;
    h.scroll_to(1, 2)?;
    h.scroll_by(3, 4)?;
    assert_eq!(
        h.mocks_mut().match_media().calls(),
        &[browser_tester::MatchMediaCall {
            query: "(min-width: 768px)".to_string()
        }]
    );
    assert_eq!(
        h.mocks_mut().dialogs().alert_messages(),
        &["hello kazu".to_string()]
    );
    assert_eq!(
        h.mocks_mut().dialogs().confirm_messages(),
        &["continue?".to_string()]
    );
    assert_eq!(
        h.mocks_mut().dialogs().prompt_messages(),
        &["name?".to_string()]
    );
    assert_eq!(h.mocks_mut().print().calls().len(), 1);
    assert_eq!(h.mocks_mut().scroll().calls().len(), 2);
    Ok(())
}

#[test]
fn stable_core_debug_view_reports_metadata() -> Result<()> {
    let h = Harness::from_html_with_url(
        "https://app.local/overview",
        "<main id='root'><p id='out'>ok</p></main>",
    )?;

    assert_eq!(h.debug().url(), "https://app.local/overview");
    assert_eq!(
        h.debug().source_html(),
        Some("<main id='root'><p id='out'>ok</p></main>")
    );
    assert_eq!(h.debug().dom_node_count(), 4);
    assert!(!h.debug().trace_enabled());
    assert!(h.debug().dump_dom().contains("<main id=\"root\">"));
    Ok(())
}

#[test]
fn stable_core_selector_dump_dom_returns_matching_node_markup() -> Result<()> {
    let h = Harness::from_html("<main id='root'><p id='out'>ok</p></main>")?;

    let snippet = h.dump_dom("#out")?;
    assert_eq!(snippet, "<p id=\"out\">ok</p>");
    Ok(())
}

#[test]
fn stable_core_keyboard_dispatch_reaches_bubbling_listeners() -> Result<()> {
    let html = "<div id='root'><input id='field'></div><p id='out'></p><script>document.getElementById('root').addEventListener('keydown', (event) => { document.getElementById('out').textContent = event.target.id + ':' + event.currentTarget.id + ':' + event.key; });</script>";

    let mut h = Harness::from_html(html)?;
    h.dispatch_keyboard(
        "#field",
        "keydown",
        browser_tester::KeyboardEventInit {
            key: "Enter".to_string(),
            code: Some("Enter".to_string()),
            ..Default::default()
        },
    )?;

    h.assert_text("#out", "field:root:Enter")?;
    Ok(())
}

#[test]
fn stable_core_negative_time_rejected() {
    let mut h = Harness::from_html("<p id='out'>ok</p>").expect("harness should build");

    let err = h.advance_time(-1).expect_err("negative time must fail");
    assert!(
        err.to_string()
            .contains("advance_time requires a non-negative delta")
    );
}
