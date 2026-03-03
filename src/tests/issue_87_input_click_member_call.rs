use super::*;

#[test]
fn issue_87_input_click_is_callable_inside_page_script_handler() -> Result<()> {
    let html = r#"
      <input id='f' type='file' />
      <button id='open'>open</button>
      <p id='out'></p>
      <script>
        const fileInput = document.getElementById('f');
        const openButton = document.getElementById('open');
        let clicked = 0;
        fileInput.addEventListener('click', () => {
          clicked += 1;
          document.getElementById('out').textContent = String(clicked);
        });
        openButton.addEventListener('click', () => {
          fileInput.click();
        });
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#open")?;
    harness.assert_text("#out", "1")?;
    Ok(())
}
