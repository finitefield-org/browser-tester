use super::*;

#[test]
fn issue_99_dispatch_paste_bubbles_to_ancestor_listener() -> Result<()> {
    let html = r#"
      <div id='parent'>
        <input id='child' />
      </div>
      <p id='out'></p>
      <script>
        document.getElementById('parent').addEventListener('paste', (event) => {
          const target = event.target;
          const isChild = target && target.id === 'child';
          const text = event.clipboardData
            ? event.clipboardData.getData('text/plain')
            : '';
          document.getElementById('out').textContent = String(isChild) + ':' + text;
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.set_clipboard_text("A001\t10.01");
    h.dispatch("#child", "paste")?;
    h.assert_text("#out", "true:A001\t10.01")?;
    Ok(())
}
