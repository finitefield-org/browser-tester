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
