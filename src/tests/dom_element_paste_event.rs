use super::*;

#[test]
fn element_paste_event_trusted_dispatch_exposes_clipboard_and_runs_default_insertion() -> Result<()>
{
    let html = r#"
        <input id='target' value='startend' />
        <p id='event'></p>
        <script>
          const target = document.getElementById('target');
          target.setSelectionRange(5, 5);
          target.addEventListener('paste', (event) => {
            document.getElementById('event').textContent = [
              event.type,
              event.clipboardData.getData('text/plain'),
              event.bubbles,
              event.cancelable,
              event.isTrusted
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.set_clipboard_text("-X-");
    h.paste("#target")?;
    h.assert_value("#target", "start-X-end")?;
    h.assert_text("#event", "paste|-X-|true|true|true")?;
    Ok(())
}

#[test]
fn element_paste_event_prevent_default_blocks_default_insertion() -> Result<()> {
    let html = r#"
        <input id='target' value='hello' />
        <p id='event'></p>
        <script>
          const target = document.getElementById('target');
          target.setSelectionRange(1, 4);
          target.addEventListener('paste', (event) => {
            event.preventDefault();
            document.getElementById('event').textContent = event.clipboardData.getData('text');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.set_clipboard_text("WORLD");
    h.paste("#target")?;
    h.assert_value("#target", "hello")?;
    h.assert_text("#event", "WORLD")?;
    Ok(())
}

#[test]
fn element_paste_event_bubbles_and_document_can_cancel_default_insertion() -> Result<()> {
    let html = r#"
        <input id='target' value='abc' />
        <p id='event'></p>
        <script>
          const target = document.getElementById('target');
          target.setSelectionRange(1, 1);
          document.addEventListener('paste', (event) => {
            event.preventDefault();
            document.getElementById('event').textContent = String(event.target === target);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.set_clipboard_text("ZZ");
    h.paste("#target")?;
    h.assert_value("#target", "abc")?;
    h.assert_text("#event", "true")?;
    Ok(())
}

#[test]
fn element_paste_event_synthetic_dispatch_does_not_apply_default_action() -> Result<()> {
    let html = r#"
        <input id='target' value='abc' />
        <p id='event'></p>
        <script>
          const target = document.getElementById('target');
          target.setSelectionRange(1, 2);
          target.addEventListener('paste', (event) => {
            document.getElementById('event').textContent = [
              event.isTrusted,
              event.bubbles,
              event.cancelable
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.set_clipboard_text("ZZ");
    h.dispatch("#target", "paste")?;
    h.assert_value("#target", "abc")?;
    h.assert_text("#event", "false|true|true")?;
    Ok(())
}

#[test]
fn element_paste_event_inserts_clipboard_text_into_contenteditable_host() -> Result<()> {
    let html = r#"
        <div id='editable' contenteditable='true'>Hello</div>
        "#;

    let mut h = Harness::from_html(html)?;
    h.set_clipboard_text(" World");
    h.paste("#editable")?;
    h.assert_text("#editable", "Hello World")?;
    Ok(())
}
