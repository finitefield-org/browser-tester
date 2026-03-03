use super::*;

#[test]
fn element_query_selector_finds_first_descendant_in_group_selector() -> Result<()> {
    let html = r#"
        <body id='root'>
          <style id='first-no-type'></style>
          <style id='second-css' type='text/css'></style>
        </body>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const el = document.body.querySelector("style[type='text/css'], style:not([type])");
            document.getElementById('result').textContent = el.id;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "first-no-type")?;
    Ok(())
}

#[test]
fn element_query_selector_supports_scope_for_direct_descendants() -> Result<()> {
    let html = r#"
        <div>
          <div id='parent'>
            <span>Love is Kind.</span>
            <span><span>Love is Patient.</span></span>
            <span><span>Love is Selfless.</span></span>
          </div>
        </div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const parentElement = document.getElementById('parent');
            const picked = parentElement.querySelector(':scope > span');
            document.getElementById('result').textContent = picked.textContent.trim();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "Love is Kind.")?;
    Ok(())
}

#[test]
fn element_query_selector_considers_full_selector_hierarchy() -> Result<()> {
    let html = r#"
        <div>
          <p id='base'>
            inside paragraph
            <span>inside span</span>
            inside paragraph
          </p>
        </div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const baseElement = document.getElementById('base');
            document.getElementById('result').textContent =
              baseElement.querySelector('div span').textContent.trim();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "inside span")?;
    Ok(())
}

#[test]
fn element_query_selector_returns_null_when_no_match_exists() -> Result<()> {
    let html = r#"
        <div id='root'>
          <span class='item'>A</span>
        </div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const root = document.getElementById('root');
            const found = root.querySelector('.missing');
            document.getElementById('result').textContent = String(found === null);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true")?;
    Ok(())
}

#[test]
fn element_query_selector_throws_syntax_error_for_invalid_selector() -> Result<()> {
    let html = r#"
        <div id='root'>
          <span class='item'>A</span>
        </div>
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('root').querySelector('div[');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#run")
        .expect_err("invalid selector should throw syntax error");
    match err {
        Error::ScriptRuntime(message) => {
            assert!(
                message.contains("SyntaxError"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected script runtime error, got: {other:?}"),
    }
    Ok(())
}
