use super::*;

#[test]
fn element_get_client_rects_returns_single_rect_for_connected_rendered_element() -> Result<()> {
    let html = r#"
        <div id='box' style='width: 120px; height: 90px;'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const rects = document.getElementById('box').getClientRects();
            let count = 0;
            let first = null;
            for (const rect of rects) {
              count += 1;
              if (!first) first = rect;
            }
            document.getElementById('result').textContent = [
              count,
              rects.length,
              first.left,
              first.top,
              first.right,
              first.bottom,
              first.width,
              first.height
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "1:1:0:0:0:0:0:0")?;
    Ok(())
}

#[test]
fn element_get_client_rects_returns_empty_for_non_rendered_elements() -> Result<()> {
    let html = r#"
        <map name='m'>
          <area id='a' shape='rect' coords='0,0,10,10' href='/go'>
        </map>
        <div id='hidden' style='display:none'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const areaLen = document.getElementById('a').getClientRects().length;
            const hiddenLen = document.getElementById('hidden').getClientRects().length;
            const detachedLen = document.createElement('div').getClientRects().length;
            document.getElementById('result').textContent = [
              areaLen,
              hiddenLen,
              detachedLen
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
fn element_get_client_rects_rejects_arguments() -> Result<()> {
    let html = r#"
        <div id='box'></div>
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('box').getClientRects(1);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("getClientRects takes no arguments"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}
