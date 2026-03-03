use super::*;

#[test]
fn global_this_exists_and_is_truthy_in_page_script() -> Result<()> {
    let html = r#"
        <div id='out'></div>
        <script>
          document.getElementById('out').textContent = String(Boolean(globalThis));
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#out", "true")?;
    Ok(())
}

#[test]
fn global_this_aliases_window_and_document() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const sameWindow = globalThis === window;
            const sameSelf = globalThis === self;
            const sameDocument = globalThis.document === document;

            globalThis.flag = 'ok';
            const reflected = window.flag + ':' + globalThis.flag;

            document.getElementById('result').textContent =
              sameWindow + ':' + sameSelf + ':' + sameDocument + '|' + reflected;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:true:true|ok:ok")?;
    Ok(())
}
