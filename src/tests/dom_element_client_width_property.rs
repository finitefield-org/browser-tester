use super::*;

#[test]
fn element_client_width_includes_padding_and_excludes_border() -> Result<()> {
    let html = r#"
        <div id='box' style='width: 100px; padding: 20px; padding-right: 10px; border: 5px solid black;'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const box = document.getElementById('box');
            document.getElementById('result').textContent = String(box.clientWidth);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "130")?;
    Ok(())
}

#[test]
fn element_client_width_is_zero_for_inline_or_no_layout_box_elements() -> Result<()> {
    let html = r#"
        <span id='inline' style='display:inline; width: 100px; padding: 20px;'></span>
        <div id='hidden' style='display:none; width: 100px; padding: 20px;'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const inline = document.getElementById('inline');
            const hidden = document.getElementById('hidden');
            const detached = document.createElement('div');
            detached.style.width = '40px';
            detached.style.padding = '10px';
            document.getElementById('result').textContent = [
              inline.clientWidth,
              hidden.clientWidth,
              detached.clientWidth
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "0:0:0")?;
    Ok(())
}

#[test]
fn document_element_client_width_uses_window_inner_width() -> Result<()> {
    let html = r#"
        <html>
          <body>
            <button id='run'>run</button>
            <p id='result'></p>
            <script>
              document.getElementById('run').addEventListener('click', () => {
                window.innerWidth = 845.9;
                document.getElementById('result').textContent =
                  String(document.documentElement.clientWidth);
              });
            </script>
          </body>
        </html>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "845")?;
    Ok(())
}
