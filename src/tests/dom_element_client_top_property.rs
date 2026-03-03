use super::*;

#[test]
fn element_client_top_returns_top_border_width() -> Result<()> {
    let html = r#"
        <div id='contained' style='margin: 1rem; border-top: 24px solid black; padding: 0 28px; overflow: auto; background: white;'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const contained = document.getElementById('contained');
            document.getElementById('result').textContent = String(contained.clientTop);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "24")?;
    Ok(())
}

#[test]
fn element_client_top_excludes_margin_and_padding() -> Result<()> {
    let html = r#"
        <div id='box' style='margin-top: 12px; padding-top: 28px; border: 4px dashed black;'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const box = document.getElementById('box');
            document.getElementById('result').textContent = String(box.clientTop);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "4")?;
    Ok(())
}

#[test]
fn element_client_top_is_zero_for_inline_or_no_layout_box() -> Result<()> {
    let html = r#"
        <span id='inline' style='display:inline; border-top: 12px solid black;'></span>
        <div id='hidden' style='display:none; border-top: 9px solid black;'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const inline = document.getElementById('inline');
            const hidden = document.getElementById('hidden');
            const detached = document.createElement('div');
            detached.style.borderTop = '8px solid black';
            document.getElementById('result').textContent = [
              inline.clientTop,
              hidden.clientTop,
              detached.clientTop
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
fn element_client_top_uses_top_value_from_border_width_shorthand() -> Result<()> {
    let html = r#"
        <div id='box' style='border-style: solid; border-width: 7px 11px 13px 17px;'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const box = document.getElementById('box');
            document.getElementById('result').textContent = String(box.clientTop);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "7")?;
    Ok(())
}
