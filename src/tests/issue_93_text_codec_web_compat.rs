use super::*;

#[test]
fn issue_93_text_encoder_encode_result_supports_uint8array_iteration_methods() -> Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        const bytes = new TextEncoder().encode('abc');
        let sum = 0;
        bytes.forEach((n) => { sum += n; });
        document.getElementById('out').textContent = String(sum);
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "294")?;
    Ok(())
}

#[test]
fn issue_93_text_decoder_is_available_as_global_constructor() -> Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        const decoder = new TextDecoder('utf-8', { fatal: false });
        document.getElementById('out').textContent =
          typeof decoder + ':' + (decoder instanceof TextDecoder);
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "object:true")?;
    Ok(())
}
