use super::*;

#[test]
fn text_encoder_encoding_is_utf8_and_read_only() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const encoder = new TextEncoder();
            const before = encoder.encoding;
            encoder.encoding = 'shift-jis';
            const after = encoder.encoding;
            document.getElementById('result').textContent = before + ':' + after;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "utf-8:utf-8")?;
    Ok(())
}

#[test]
fn text_encoder_encode_uses_utf8_and_defaults_to_empty_string() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const encoder = new TextEncoder();
            const euro = Array.from(encoder.encode('€')).join(',');
            const emptyLength = encoder.encode().length;
            document.getElementById('result').textContent = euro + ':' + emptyLength;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "226,130,172:0")?;
    Ok(())
}

#[test]
fn text_encoder_encode_into_reports_read_and_written_and_writes_bytes() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const encoder = new TextEncoder();
            const dest = new Uint8Array(10);
            const status = encoder.encodeInto('A€', dest);
            const written = Array.from(dest).slice(0, status.written).join(',');
            document.getElementById('result').textContent = [
              status.read,
              status.written,
              written
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "2:4:65,226,130,172")?;
    Ok(())
}

#[test]
fn text_encoder_encode_into_stops_before_incomplete_code_point_and_validates_destination_type(
) -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const encoder = new TextEncoder();
            const small = new Uint8Array(4);
            const status = encoder.encodeInto('🙂A', small);
            const bytes = Array.from(small).join(',');

            let typeError = false;
            try {
              encoder.encodeInto('x', new Int8Array(2));
            } catch (e) {
              typeError = String(e).includes('Uint8Array');
            }

            document.getElementById('result').textContent = [
              status.read,
              status.written,
              bytes,
              typeError
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "2:4:240,159,153,130:true")?;
    Ok(())
}
