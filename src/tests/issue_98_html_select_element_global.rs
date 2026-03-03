use super::*;

#[test]
fn issue_98_html_select_element_global_exists_and_instanceof_works() -> Result<()> {
    let html = r#"
      <select id='s'><option value='a'>a</option><option value='b'>b</option></select>
      <p id='out'></p>
      <script>
        const s = document.getElementById('s');
        try {
          const info = typeof HTMLSelectElement;
          const isSelect = s instanceof HTMLSelectElement;
          document.getElementById('out').textContent = info + ':' + String(isSelect);
        } catch (error) {
          document.getElementById('out').textContent =
            'ERR:' + String(error && error.message ? error.message : error);
        }
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "function:true")?;
    Ok(())
}
