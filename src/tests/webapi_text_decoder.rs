use super::*;

#[test]
fn text_decoder_default_utf8_decodes_multibyte_text() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const decoder = new TextDecoder();
            const encoded = new Uint8Array([240, 160, 174, 183]);
            document.getElementById('result').textContent = decoder.decode(encoded);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "𠮷")?;
    Ok(())
}

#[test]
fn text_decoder_windows_1251_decodes_russian_text() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const decoder = new TextDecoder('windows-1251');
            const encoded = new Uint8Array([
              207, 240, 232, 226, 229, 242, 44, 32, 236, 232, 240, 33
            ]);
            document.getElementById('result').textContent = decoder.decode(encoded);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "Привет, мир!")?;
    Ok(())
}

#[test]
fn text_decoder_properties_reflect_options_and_are_read_only() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const decoder = new TextDecoder('utf-8', { fatal: true, ignoreBOM: true });
            const before = [decoder.encoding, decoder.fatal, decoder.ignoreBOM].join(':');
            decoder.encoding = 'windows-1251';
            decoder.fatal = false;
            decoder.ignoreBOM = false;
            const after = [decoder.encoding, decoder.fatal, decoder.ignoreBOM].join(':');
            document.getElementById('result').textContent = before + '|' + after;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "utf-8:true:true|utf-8:true:true")?;
    Ok(())
}

#[test]
fn text_decoder_decode_supports_array_buffer_and_default_empty_input() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const decoder = new TextDecoder();
            const bytes = new Uint8Array([65, 66, 67]);
            const fromBuffer = decoder.decode(bytes.buffer);
            const fromEmpty = decoder.decode();
            document.getElementById('result').textContent = fromBuffer + ':' + fromEmpty.length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "ABC:0")?;
    Ok(())
}

#[test]
fn text_decoder_fatal_and_ignore_bom_options_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const fatal = new TextDecoder('utf-8', { fatal: true });
            let fatalError = false;
            try {
              fatal.decode(new Uint8Array([0xFF]));
            } catch (e) {
              fatalError = String(e).includes('invalid UTF-8');
            }

            const keepBom = new TextDecoder('utf-8', { ignoreBOM: true });
            const stripBom = new TextDecoder('utf-8');
            const payload = new Uint8Array([0xEF, 0xBB, 0xBF, 0x41]);
            const kept = keepBom.decode(payload);
            const stripped = stripBom.decode(payload);

            document.getElementById('result').textContent = [
              fatalError,
              kept.length,
              kept.codePointAt(0),
              kept.codePointAt(1),
              stripped.length,
              stripped.codePointAt(0)
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:2:65279:65:1:65")?;
    Ok(())
}
