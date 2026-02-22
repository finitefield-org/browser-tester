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
