use super::*;

#[test]
fn issue_101_template_literal_interpolation_uses_plain_object_property_without_typed_array_error()
-> Result<()> {
    let html = r#"
      <p id='out'></p>
      <p id='err'></p>
      <script>
        const text = { bytesLabel: "Input bytes" };
        const state = { byteLength: 3 };
        try {
          document.getElementById('out').textContent = `${text.bytesLabel}: ${state.byteLength}`;
        } catch (error) {
          document.getElementById('err').textContent =
            String(error && error.message ? error.message : error);
        }
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "Input bytes: 3")?;
    harness.assert_text("#err", "")?;
    Ok(())
}
