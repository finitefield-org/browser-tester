use super::*;

#[test]
fn dispatch_paste_exposes_clipboard_data_payload() -> Result<()> {
    let html = r#"
      <input id='i' />
      <p id='out'></p>
      <script>
        const input = document.getElementById('i');
        input.addEventListener('paste', (event) => {
          const val = event.clipboardData && event.clipboardData.getData
            ? event.clipboardData.getData('text/plain')
            : '';
          document.getElementById('out').textContent = val || '(empty)';
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.set_clipboard_text("A001\t10.01\t9.99");
    h.dispatch("#i", "paste")?;
    h.assert_text("#out", "A001\t10.01\t9.99")?;
    Ok(())
}

#[test]
fn dispatch_paste_get_data_returns_empty_string_for_unknown_format() -> Result<()> {
    let html = r#"
      <input id='i' />
      <p id='out'></p>
      <script>
        document.getElementById('i').addEventListener('paste', (event) => {
          document.getElementById('out').textContent = [
            event.clipboardData.getData('application/json') === '',
            event.clipboardData.getData('text')
          ].join(':');
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.set_clipboard_text("hello");
    h.dispatch("#i", "paste")?;
    h.assert_text("#out", "true:hello")?;
    Ok(())
}
