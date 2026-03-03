use super::*;

#[test]
fn element_get_bounding_client_rect_returns_dom_rect_like_object() -> Result<()> {
    let html = r#"
        <div id='box' style='width: 120px; height: 90px;'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const box = document.getElementById('box');
            const rect = box.getBoundingClientRect();
            document.getElementById('result').textContent = [
              typeof rect === 'object',
              rect.x,
              rect.y,
              rect.left,
              rect.top,
              rect.right,
              rect.bottom,
              rect.width,
              rect.height,
              rect.right === rect.left + rect.width,
              rect.bottom === rect.top + rect.height
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:0:0:0:0:0:0:0:0:true:true")?;
    Ok(())
}

#[test]
fn element_get_bounding_client_rect_reflects_document_scroll_offset() -> Result<()> {
    let html = r#"
        <div id='box'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const box = document.getElementById('box');
            box.scrollTo(25, 40);
            const rect = box.getBoundingClientRect();
            document.getElementById('result').textContent = [
              rect.left,
              rect.top,
              rect.x,
              rect.y,
              rect.right,
              rect.bottom
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "-25:-40:-25:-40:-25:-40")?;
    Ok(())
}

#[test]
fn element_get_bounding_client_rect_rejects_arguments() -> Result<()> {
    let html = r#"
        <div id='box'></div>
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('box').getBoundingClientRect(1);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("getBoundingClientRect takes no arguments"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}
