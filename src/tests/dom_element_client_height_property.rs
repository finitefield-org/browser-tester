use super::*;

#[test]
fn element_client_height_includes_padding_and_excludes_border() -> Result<()> {
    let html = r#"
        <div id='box' style='height: 100px; padding: 20px; padding-bottom: 10px; border: 5px solid black;'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const box = document.getElementById('box');
            document.getElementById('result').textContent = String(box.clientHeight);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "130")?;
    Ok(())
}

#[test]
fn element_client_height_is_zero_for_no_layout_box_elements() -> Result<()> {
    let html = r#"
        <div id='hidden' style='display:none; height: 100px; padding: 20px;'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const hidden = document.getElementById('hidden');
            const detached = document.createElement('div');
            detached.style.height = '40px';
            detached.style.padding = '10px';
            document.getElementById('result').textContent = [
              hidden.clientHeight,
              detached.clientHeight
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "0:0")?;
    Ok(())
}

#[test]
fn document_element_client_height_uses_window_inner_height() -> Result<()> {
    let html = r#"
        <html>
          <body>
            <button id='run'>run</button>
            <p id='result'></p>
            <script>
              document.getElementById('run').addEventListener('click', () => {
                window.innerHeight = 612.75;
                document.getElementById('result').textContent =
                  String(document.documentElement.clientHeight);
              });
            </script>
          </body>
        </html>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "612")?;
    Ok(())
}
