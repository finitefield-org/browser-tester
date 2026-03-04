use super::*;

#[test]
fn canvas_width_height_defaults_and_attribute_reflection_work() -> Result<()> {
    let html = r#"
        <canvas id='canvas'>Fallback text</canvas>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const canvas = document.getElementById('canvas');
            const initial =
              canvas.width + 'x' + canvas.height + ':' +
              (canvas.getAttribute('width') === null) + ':' +
              (canvas.getAttribute('height') === null);

            canvas.width = 120;
            canvas.height = 80;
            const assigned =
              canvas.width + 'x' + canvas.height + ':' +
              canvas.getAttribute('width') + ':' +
              canvas.getAttribute('height');

            canvas.setAttribute('width', 'oops');
            canvas.setAttribute('height', '-5');
            const normalized = canvas.width + 'x' + canvas.height;

            const fallback = canvas.textContent.trim();

            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + normalized + '|' + fallback;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "300x150:true:true|120x80:120:80|300x150|Fallback text",
    )?;
    Ok(())
}

#[test]
fn canvas_direct_dom_query_width_height_assignment_works() -> Result<()> {
    let html = r#"
        <canvas id='canvas'></canvas>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('canvas').width = 640;
            document.getElementById('canvas').height = 360;
            document.getElementById('result').textContent =
              document.getElementById('canvas').width + 'x' +
              document.getElementById('canvas').height + ':' +
              document.getElementById('canvas').getAttribute('width') + ':' +
              document.getElementById('canvas').getAttribute('height');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "640x360:640:360")?;
    Ok(())
}

#[test]
fn canvas_get_context_2d_supports_fill_style_and_fill_rect_calls() -> Result<()> {
    let html = r#"
        <canvas id='canvas' width='120' height='120'></canvas>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const canvas = document.getElementById('canvas');
            const first = canvas.getContext('2d', { alpha: false });
            first.fillStyle = 'green';
            first.fillRect(10, 10, 100, 100);
            const second = canvas.getContext('2d');
            const attrs = second.getContextAttributes();
            const noWebGl = canvas.getContext('webgl') === null;
            document.getElementById('result').textContent =
              (first === second) + '|' + second.fillStyle + '|' + attrs.alpha + '|' + noWebGl;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true|green|false|true")?;
    Ok(())
}

#[test]
fn canvas_get_context_allows_2d_after_unsupported_context_request() -> Result<()> {
    let html = r#"
        <canvas id='canvas' width='120' height='120'></canvas>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const canvas = document.getElementById('canvas');
            const webgl = canvas.getContext('webgl');
            const first2d = canvas.getContext('2d');
            const second2d = canvas.getContext('2d');
            const bitmap = canvas.getContext('bitmaprenderer');
            document.getElementById('result').textContent =
              (webgl === null) + '|' +
              (first2d !== null) + '|' +
              (first2d === second2d) + '|' +
              (bitmap === null);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true|true|true|true")?;
    Ok(())
}

#[test]
fn canvas_get_context_throws_after_transfer_control_to_offscreen() -> Result<()> {
    let html = r#"
        <canvas id='canvas' width='120' height='120'></canvas>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const canvas = document.getElementById('canvas');
            const offscreen = canvas.transferControlToOffscreen();
            let threw = false;
            try {
              canvas.getContext('2d');
            } catch (err) {
              threw = String(err).includes('InvalidStateError');
            }
            document.getElementById('result').textContent =
              (offscreen !== null) + '|' + threw;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true|true")?;
    Ok(())
}

#[test]
fn canvas_transfer_control_to_offscreen_throws_after_context_creation() -> Result<()> {
    let html = r#"
        <canvas id='canvas' width='120' height='120'></canvas>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const canvas = document.getElementById('canvas');
            const ctx = canvas.getContext('2d');
            let threw = false;
            try {
              canvas.transferControlToOffscreen();
            } catch (err) {
              threw = String(err).includes('InvalidStateError');
            }
            document.getElementById('result').textContent =
              (ctx !== null) + '|' + threw;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true|true")?;
    Ok(())
}

#[test]
fn canvas_to_data_url_returns_data_url_prefixes() -> Result<()> {
    let html = r#"
        <canvas id='canvas'></canvas>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const canvas = document.getElementById('canvas');
            const png = canvas.toDataURL();
            const jpeg = canvas.toDataURL('image/jpeg');
            document.getElementById('result').textContent =
              png.startsWith('data:image/png;base64,') + '|' +
              jpeg.startsWith('data:image/jpeg;base64,');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true|true")?;
    Ok(())
}

#[test]
fn canvas_to_blob_supports_type_quality_and_fallback() -> Result<()> {
    let html = r#"
        <canvas id='canvas'></canvas>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const canvas = document.getElementById('canvas');
            const logs = [];
            const returned = canvas.toBlob((blob) => {
              logs.push(blob.type);
              logs.push(blob.size > 0);
              logs.push(URL.createObjectURL(blob).startsWith('blob:bt-'));
            });
            canvas.toBlob((blob) => {
              logs.push(blob.type);
            }, 'image/jpeg', 0.95);
            canvas.toBlob((blob) => {
              logs.push(blob.type);
            }, 'application/json');
            document.getElementById('result').textContent =
              (returned === undefined) + '|' + logs.join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true|image/png|true|true|image/jpeg|image/png")?;
    Ok(())
}

#[test]
fn canvas_to_blob_requires_callable_callback() -> Result<()> {
    let html = r#"
        <canvas id='canvas'></canvas>
        <button id='run' type='button'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('canvas').toBlob(null);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("toBlob callback must be callable"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}
