use super::*;

#[test]
fn issue_88_string_from_char_code_apply_accepts_uint8_array_arguments() -> Result<()> {
    let html = r#"
      <p id='out'></p>
      <p id='err'></p>
      <script>
        try {
          const part = new Uint8Array([65, 66, 67]);
          const result = String.fromCharCode.apply(null, part);
          document.getElementById('out').textContent = result;
        } catch (error) {
          document.getElementById('err').textContent =
            String(error && error.message ? error.message : error);
        }
      </script>
    "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#out", "ABC")?;
    h.assert_text("#err", "")?;
    Ok(())
}
