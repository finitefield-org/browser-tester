use super::*;

#[test]
fn document_active_element_defaults_to_body_when_unfocused() -> Result<()> {
    let html = r#"
        <html id='doc'>
          <head></head>
          <body id='body'>
            <p id='result'></p>
            <script>
              const active = document.activeElement;
              document.getElementById('result').textContent = active ? active.id : 'none';
            </script>
          </body>
        </html>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "body")?;
    Ok(())
}

#[test]
fn document_active_element_defaults_to_document_element_without_body() -> Result<()> {
    let html = r#"
        <html id='doc'>
          <head></head>
          <p id='result'></p>
          <script>
            const active = document.activeElement;
            document.getElementById('result').textContent = active ? active.id : 'none';
          </script>
        </html>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "doc")?;
    Ok(())
}

#[test]
fn document_active_element_returns_focused_element() -> Result<()> {
    let html = r#"
        <input id='field'>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const field = document.getElementById('field');
            field.focus();
            const active = document.activeElement;
            document.getElementById('result').textContent = active ? active.id : 'none';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "field")?;
    Ok(())
}

#[test]
fn document_active_element_ignores_hidden_and_disconnected_focus_targets() -> Result<()> {
    let html = r#"
        <body id='body'>
          <div id='hidden' hidden tabindex='0'></div>
          <input id='field'>
          <button id='run'>run</button>
          <p id='result'></p>
          <script>
            document.getElementById('run').addEventListener('click', () => {
            const hidden = document.getElementById('hidden');
            const field = document.getElementById('field');
            const detached = document.createElement('input');

            hidden.focus();
            const activeAfterHidden = document.activeElement;
            const first = activeAfterHidden ? activeAfterHidden.id : 'none';

            detached.focus();
            const activeAfterDetached = document.activeElement;
            const second = activeAfterDetached ? activeAfterDetached.id : 'none';

            field.focus();
            field.remove();
            const activeAfterRemove = document.activeElement;
            const third = activeAfterRemove ? activeAfterRemove.id : 'none';
            const hasFocus = document.hasFocus();

            document.getElementById('result').textContent =
              [first, second, third, String(hasFocus)].join(':');
            });
          </script>
        </body>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "body:body:body:false")?;
    Ok(())
}
