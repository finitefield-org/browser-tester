use super::*;

#[test]
fn issue_95_window_add_event_listener_does_not_throw_and_script_continues() -> Result<()> {
    let html = r#"
        <p id='out'></p>
        <script>
          window.addEventListener('resize', () => {
            document.getElementById('out').textContent = 'resized';
          });
          document.getElementById('out').textContent = 'ready';
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#out", "ready")?;
    Ok(())
}

#[test]
fn issue_95_window_dispatch_event_invokes_registered_listener() -> Result<()> {
    let html = r#"
        <p id='out'></p>
        <script>
          window.addEventListener('resize', () => {
            document.getElementById('out').textContent = 'resized';
          });
          window.dispatchEvent(new Event('resize'));
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#out", "resized")?;
    Ok(())
}
