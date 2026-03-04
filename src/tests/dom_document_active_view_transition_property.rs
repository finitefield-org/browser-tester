use super::*;

#[test]
fn document_active_view_transition_defaults_to_null() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          document.getElementById('result').textContent =
            String(document.activeViewTransition === null);
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "true")?;
    Ok(())
}

#[test]
fn document_active_view_transition_is_falsy_when_no_transition_is_active() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const state = document.activeViewTransition ? 'active' : 'none';
            document.getElementById('result').textContent = state;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "none")?;
    Ok(())
}

#[test]
fn document_active_view_transition_is_read_only() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.activeViewTransition = null;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#run")
        .expect_err("activeViewTransition should be read-only");
    match err {
        Error::ScriptRuntime(message) => {
            assert!(message.contains("activeViewTransition is read-only"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
    Ok(())
}
