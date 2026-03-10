use super::*;

#[test]
fn text_encoder_stream_exposes_encoding_readable_and_writable() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const stream = new TextEncoderStream();
            document.getElementById('result').textContent = [
              stream.encoding,
              String(stream.readable),
              String(stream.writable),
              window.TextEncoderStream === TextEncoderStream,
              stream.readable === stream.readable,
              stream.writable === stream.writable
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "utf-8:[object ReadableStream]:[object WritableStream]:true:true:true",
    )?;
    Ok(())
}

#[test]
fn text_encoder_stream_properties_are_read_only() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const stream = new TextEncoderStream();
            const originalReadable = stream.readable;
            const originalWritable = stream.writable;
            stream.encoding = 'windows-1251';
            stream.readable = null;
            stream.writable = null;
            document.getElementById('result').textContent = [
              stream.encoding,
              stream.readable === originalReadable,
              stream.writable === originalWritable
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "utf-8:true:true")?;
    Ok(())
}

#[test]
fn text_encoder_stream_constructor_rejects_arguments() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            let threw = false;
            try {
              new TextEncoderStream(1);
            } catch (e) {
              threw = String(e).includes('does not take arguments');
            }
            document.getElementById('result').textContent = String(threw);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true")?;
    Ok(())
}

#[test]
fn text_encoder_stream_has_branded_prototype_surface_and_validates_receiver() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const stream = new TextEncoderStream();
            const readableGetter = Object.getOwnPropertyDescriptor(TextEncoderStream.prototype, 'readable').get;
            let receiverError = false;
            try {
              readableGetter.call({});
            } catch (e) {
              receiverError = String(e).includes('incompatible receiver');
            }
            document.getElementById('result').textContent = [
              Object.prototype.toString.call(stream),
              TextEncoderStream.prototype[Symbol.toStringTag],
              String(Object.getPrototypeOf(stream) === TextEncoderStream.prototype),
              String(stream.constructor === TextEncoderStream),
              String(Object.getOwnPropertyDescriptor(TextEncoderStream.prototype, 'encoding').get.call(stream)),
              String(Object.prototype.toString.call(readableGetter.call(stream))),
              String(receiverError)
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "[object TextEncoderStream]|TextEncoderStream|true|true|utf-8|[object ReadableStream]|true",
    )?;
    Ok(())
}
