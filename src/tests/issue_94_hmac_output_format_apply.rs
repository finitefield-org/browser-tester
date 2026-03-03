use super::*;

#[test]
fn issue_94_output_format_switch_using_string_from_char_code_apply_updates_result() -> Result<()> {
    let html = r#"
      <select id='format'>
        <option value='hex'>hex</option>
        <option value='base64'>base64</option>
      </select>
      <p id='out'></p>
      <p id='err'></p>
      <script>
        const bytes = new Uint8Array([222, 173, 190, 239]);

        function toHex(values) {
          return Array.from(values)
            .map((n) => n.toString(16).padStart(2, '0'))
            .join('');
        }

        function toBase64(values) {
          return btoa(String.fromCharCode.apply(null, values));
        }

        function updateOutput() {
          try {
            const selected = document.getElementById('format').value;
            document.getElementById('out').textContent =
              selected === 'hex' ? toHex(bytes) : toBase64(bytes);
            document.getElementById('err').textContent = '';
          } catch (error) {
            document.getElementById('err').textContent =
              String(error && error.message ? error.message : error);
          }
        }

        document.getElementById('format').addEventListener('change', updateOutput);
        updateOutput();
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.assert_text("#out", "deadbeef")?;
    h.assert_text("#err", "")?;

    h.set_select_value("#format", "base64")?;
    h.assert_text("#out", "3q2+7w==")?;
    h.assert_text("#err", "")?;
    Ok(())
}
