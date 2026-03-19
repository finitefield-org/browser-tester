use browser_tester::{
    Error, Harness, LocationNavigation, LocationNavigationKind, MockFile, Result,
};

#[test]
fn stable_core_actions_and_assertions_work_together() -> Result<()> {
    let html = r#"
      <button id='run' type='button'>run</button>
      <form id='form'>
        <input id='name' />
        <input id='agree' type='checkbox' />
        <button id='submitter' type='submit'>submit</button>
      </form>
      <p id='clicked'></p>
      <p id='submitted'></p>
      <script>
        document.getElementById('run').addEventListener('click', () => {
          document.getElementById('clicked').textContent = 'clicked';
        });
        document.getElementById('form').addEventListener('submit', (event) => {
          event.preventDefault();
          document.getElementById('submitted').textContent = [
            document.getElementById('name').value,
            String(document.getElementById('agree').checked)
          ].join('|');
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.type_text("#name", "Alice")?;
    h.set_checked("#agree", true)?;
    h.click("#run")?;
    h.submit("#form")?;

    h.assert_text("#clicked", "clicked")?;
    h.assert_text("#submitted", "Alice|true")?;
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
    match err {
        Error::SelectorNotFound(selector) => {
            assert_eq!(selector, "#missing");
        }
        other => panic!("unexpected error: {other:?}"),
    }

    Ok(())
}

#[test]
fn stable_core_constructors_and_time_controls_work() -> Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        const out = document.getElementById('out');
        out.textContent = [
          location.href,
          localStorage.getItem('token'),
          localStorage.getItem('mode')
        ].join('|');
        setTimeout(() => {
          out.textContent += '|done';
        }, 25);
      </script>
    "#;

    let mut h = Harness::from_html_with_url_and_local_storage(
        "https://app.local/start",
        html,
        &[("token", "seed"), ("mode", "debug")],
    )?;

    h.assert_text("#out", "https://app.local/start|seed|debug")?;
    h.advance_time(24)?;
    h.assert_text("#out", "https://app.local/start|seed|debug")?;
    h.advance_time(1)?;
    h.assert_text("#out", "https://app.local/start|seed|debug|done")?;
    Ok(())
}

#[test]
fn stable_test_mock_fetch_contract_is_direct() -> Result<()> {
    let html = r#"
      <button id='run'>run</button>
      <p id='out'></p>
      <script>
        document.getElementById('run').addEventListener('click', () => {
          fetch('https://app.local/api/message')
            .then((res) => res.text())
            .then((text) => {
              document.getElementById('out').textContent = text;
            });
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.set_fetch_mock("https://app.local/api/message", "hello");
    h.click("#run")?;

    h.assert_text("#out", "hello")?;
    assert_eq!(
        h.take_fetch_calls(),
        vec!["https://app.local/api/message".to_string()]
    );
    Ok(())
}

#[test]
fn stable_test_mock_clipboard_contract_is_direct() -> Result<()> {
    let mut h = Harness::from_html("<p id='out'></p>")?;
    h.set_clipboard_text("seeded");

    assert_eq!(h.clipboard_text(), "seeded");
    Ok(())
}

#[test]
fn stable_test_mock_location_contract_is_direct() -> Result<()> {
    let html = r#"
      <a id='go' href='https://app.local/next'>next</a>
    "#;

    let mut h = Harness::from_html_with_url("https://app.local/start", html)?;
    h.set_location_mock_page("https://app.local/next", "<p id='msg'>next page</p>");
    h.click("#go")?;

    h.assert_text("#msg", "next page")?;
    assert_eq!(
        h.take_location_navigations(),
        vec![LocationNavigation {
            kind: LocationNavigationKind::Assign,
            from: "https://app.local/start".to_string(),
            to: "https://app.local/next".to_string(),
        }]
    );
    Ok(())
}

#[test]
fn stable_test_mock_file_input_contract_is_direct() -> Result<()> {
    let html = r#"
      <input id='upload' type='file' multiple />
      <p id='out'></p>
      <script>
        const upload = document.getElementById('upload');
        upload.addEventListener('change', () => {
          document.getElementById('out').textContent = [
            upload.value,
            upload.files.length,
            Array.from(upload.files).map((file) => file.name).join(',')
          ].join('|');
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.set_input_files(
        "#upload",
        &[
            MockFile::new("first.txt").with_text("alpha"),
            MockFile::new("second.txt").with_text("beta"),
        ],
    )?;

    h.assert_text("#out", "C:\\fakepath\\first.txt|2|first.txt,second.txt")?;
    Ok(())
}

#[test]
fn stable_core_scheduler_controls_are_direct() -> Result<()> {
    let html = r#"
      <button id='run'>run</button>
      <p id='out'></p>
      <script>
        document.getElementById('run').addEventListener('click', () => {
          const out = document.getElementById('out');
          out.textContent = 'scheduled';
          setTimeout(() => {
            out.textContent += '|timeout';
          }, 5);
          setInterval(() => {
            out.textContent += '|interval';
          }, 10);
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    assert_eq!(h.now_ms(), 0);
    assert!(h.pending_timers().is_empty());

    h.click("#run")?;

    let pending = h.pending_timers();
    assert_eq!(pending.len(), 2);
    assert_eq!(pending[0].due_at, 5);
    assert_eq!(pending[0].interval_ms, None);
    assert_eq!(pending[1].due_at, 10);
    assert_eq!(pending[1].interval_ms, Some(10));
    assert!(!h.run_next_due_timer()?);
    assert!(h.clear_timer(pending[1].id));
    assert_eq!(h.pending_timers().len(), 1);

    assert!(h.run_next_timer()?);
    assert_eq!(h.now_ms(), 5);
    h.assert_text("#out", "scheduled|timeout")?;
    assert!(!h.run_next_timer()?);
    assert_eq!(h.clear_all_timers(), 0);
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
    let html = r#"
      <button id='run'>run</button>
      <p id='out'></p>
      <script>
        document.getElementById('run').addEventListener('click', () => {
          const media = matchMedia('(min-width: 768px)');
          const accepted = confirm('continue?');
          const name = prompt('name?', 'guest');
          alert('hello ' + name);
          print();
          document.getElementById('out').textContent = [
            String(media.matches),
            media.media,
            String(accepted),
            String(name)
          ].join('|');
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.set_match_media_mock("(min-width: 768px)", true);
    h.enqueue_confirm_response(true);
    h.enqueue_prompt_response(Some("kazu"));
    h.click("#run")?;

    h.assert_text("#out", "true|(min-width: 768px)|true|kazu")?;
    assert_eq!(
        h.take_match_media_calls(),
        vec!["(min-width: 768px)".to_string()]
    );
    assert_eq!(h.take_alert_messages(), vec!["hello kazu".to_string()]);
    assert_eq!(h.take_print_call_count(), 1);

    h.clear_match_media_mocks();
    h.set_default_match_media_matches(false);
    h.set_default_confirm_response(false);
    h.set_default_prompt_response(Some("guest2"));
    h.click("#run")?;

    h.assert_text("#out", "false|(min-width: 768px)|false|guest2")?;
    assert_eq!(
        h.take_match_media_calls(),
        vec!["(min-width: 768px)".to_string()]
    );
    assert_eq!(h.take_alert_messages(), vec!["hello guest2".to_string()]);
    assert_eq!(h.take_print_call_count(), 1);
    Ok(())
}

#[test]
fn stable_test_mock_clipboard_error_controls_are_direct() -> Result<()> {
    let html = r#"
      <button id='run'>run</button>
      <p id='out'></p>
      <script>
        document.getElementById('run').addEventListener('click', () => {
          navigator.clipboard.writeText('saved')
            .then(() => navigator.clipboard.readText())
            .then((value) => {
              document.getElementById('out').textContent = 'ok:' + value;
            })
            .catch((reason) => {
              document.getElementById('out').textContent = 'err:' + String(reason);
            });
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.set_clipboard_write_error(Some("WriteBlocked"));
    h.click("#run")?;
    h.assert_text("#out", "err:WriteBlocked")?;

    h.clear_clipboard_errors();
    h.click("#run")?;
    h.assert_text("#out", "ok:saved")?;
    assert_eq!(h.clipboard_text(), "saved");
    Ok(())
}
