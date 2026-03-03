use super::*;

#[test]
fn text_decoder_stream_exposes_properties_and_streams() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const stream = new TextDecoderStream('windows-1251', { fatal: true, ignoreBOM: true });
            document.getElementById('result').textContent = [
              stream.encoding,
              stream.fatal,
              stream.ignoreBOM,
              String(stream.readable),
              String(stream.writable),
              window.TextDecoderStream === TextDecoderStream
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "windows-1251:true:true:[object ReadableStream]:[object WritableStream]:true",
    )?;
    Ok(())
}

#[test]
fn text_decoder_stream_properties_are_read_only() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const stream = new TextDecoderStream();
            const originalReadable = stream.readable;
            const originalWritable = stream.writable;
            stream.encoding = 'windows-1251';
            stream.fatal = true;
            stream.ignoreBOM = true;
            stream.readable = null;
            stream.writable = null;
            document.getElementById('result').textContent = [
              stream.encoding,
              stream.fatal,
              stream.ignoreBOM,
              stream.readable === originalReadable,
              stream.writable === originalWritable
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "utf-8:false:false:true:true")?;
    Ok(())
}

#[test]
fn text_decoder_stream_constructor_validates_arguments() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            let tooManyArgs = false;
            let unsupportedEncoding = false;
            try {
              new TextDecoderStream('utf-8', {}, 'extra');
            } catch (e) {
              tooManyArgs = String(e).includes('supports up to two arguments');
            }
            try {
              new TextDecoderStream('shift_jis');
            } catch (e) {
              unsupportedEncoding = String(e).includes('unsupported encoding label');
            }
            document.getElementById('result').textContent =
              String(tooManyArgs && unsupportedEncoding);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true")?;
    Ok(())
}
