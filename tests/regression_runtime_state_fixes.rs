use browser_tester::{Error, Harness, Result};

#[test]
fn listener_error_keeps_state_changes_before_throw() -> Result<()> {
    let html = r#"
        <button id='boom'>boom</button>
        <button id='check'>check</button>
        <p id='result'></p>
        <script>
          let x = 0;
          document.getElementById('boom').addEventListener('click', () => {
            x = 1;
            unknown_fn();
          });
          document.getElementById('check').addEventListener('click', () => {
            document.getElementById('result').textContent = String(x);
          });
        </script>
        "#;

    let mut harness = Harness::from_html(html)?;
    match harness.click("#boom") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("unknown variable: unknown_fn"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected click to fail with runtime error, got: {other:?}"),
    }

    harness.click("#check")?;
    harness.assert_text("#result", "1")?;
    Ok(())
}

#[test]
fn scheduling_timers_at_i64_max_now_does_not_overflow() -> Result<()> {
    let html = r#"
        <button id='timeout'>timeout</button>
        <button id='interval'>interval</button>
        <p id='result'></p>
        <script>
          let intervalId = 0;
          document.getElementById('timeout').addEventListener('click', () => {
            setTimeout(() => {
              document.getElementById('result').textContent += 't';
            }, 1);
          });
          document.getElementById('interval').addEventListener('click', () => {
            intervalId = setInterval(() => {
              document.getElementById('result').textContent += 'i';
              clearInterval(intervalId);
            }, 1);
          });
        </script>
        "#;

    let mut harness = Harness::from_html(html)?;
    harness.advance_time(i64::MAX)?;
    harness.click("#timeout")?;
    harness.click("#interval")?;
    assert_eq!(harness.pending_timers().len(), 2);
    assert_eq!(harness.run_due_timers()?, 2);
    harness.assert_text("#result", "ti")?;
    Ok(())
}
