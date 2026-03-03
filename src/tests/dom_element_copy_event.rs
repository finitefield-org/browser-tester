use super::*;

#[test]
fn element_copy_event_default_action_copies_selected_text_from_input() -> Result<()> {
    let html = r#"
        <input id='target' value='Alpha Beta' />
        <p id='event'></p>
        <script>
          const target = document.getElementById('target');
          target.setSelectionRange(0, 5);
          target.addEventListener('copy', (event) => {
            document.getElementById('event').textContent = [
              event.type,
              event.bubbles,
              event.cancelable,
              event.isTrusted,
              event.clipboardData.getData('text/plain') === ''
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.set_clipboard_text("original");
    h.copy("#target")?;
    assert_eq!(h.clipboard_text(), "Alpha");
    h.assert_text("#event", "copy|true|true|true|true")?;
    Ok(())
}

#[test]
fn element_copy_event_prevent_default_and_set_data_overrides_clipboard_text() -> Result<()> {
    let html = r#"
        <input id='target' value='Alpha Beta' />
        <script>
          const target = document.getElementById('target');
          target.setSelectionRange(0, 5);
          document.addEventListener('copy', (event) => {
            event.preventDefault();
            event.clipboardData.setData('text/plain', 'CUSTOM-COPY');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.set_clipboard_text("original");
    h.copy("#target")?;
    assert_eq!(h.clipboard_text(), "CUSTOM-COPY");
    Ok(())
}

#[test]
fn element_copy_event_prevent_default_without_set_data_keeps_existing_clipboard_text() -> Result<()>
{
    let html = r#"
        <input id='target' value='Alpha Beta' />
        <script>
          const target = document.getElementById('target');
          target.setSelectionRange(0, 5);
          target.addEventListener('copy', (event) => {
            event.preventDefault();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.set_clipboard_text("original");
    h.copy("#target")?;
    assert_eq!(h.clipboard_text(), "original");
    Ok(())
}

#[test]
fn element_copy_event_synthetic_dispatch_does_not_change_clipboard_text() -> Result<()> {
    let html = r#"
        <input id='target' value='Alpha Beta' />
        <p id='event'></p>
        <script>
          const target = document.getElementById('target');
          target.setSelectionRange(0, 5);
          target.addEventListener('copy', (event) => {
            event.preventDefault();
            event.clipboardData.setData('text/plain', 'SHOULD_NOT_APPLY');
            document.getElementById('event').textContent = String(event.isTrusted);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.set_clipboard_text("original");
    h.dispatch("#target", "copy")?;
    assert_eq!(h.clipboard_text(), "original");
    h.assert_text("#event", "false")?;
    Ok(())
}
