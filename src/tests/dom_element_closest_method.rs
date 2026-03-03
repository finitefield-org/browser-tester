use super::*;

#[test]
fn element_closest_matches_mdn_examples() -> Result<()> {
    let html = r#"
        <article>
          <div id='div-01'>
            Here is div-01
            <div id='div-02'>
              Here is div-02
              <div id='div-03'>Here is div-03</div>
            </div>
          </div>
        </article>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const el = document.getElementById('div-03');
            const byId = el.closest('#div-02');
            const divInDiv = el.closest('div div');
            const articleChildDiv = el.closest('article > div');
            const notDiv = el.closest(':not(div)');
            document.getElementById('result').textContent = [
              byId.id,
              divInDiv.id,
              articleChildDiv.id,
              notDiv.tagName
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "div-02:div-03:div-01:ARTICLE")?;
    Ok(())
}

#[test]
fn element_closest_returns_null_when_no_matching_ancestor() -> Result<()> {
    let html = r#"
        <div id='root'>
          <button id='btn'>run</button>
        </div>
        <button id='run'>go</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const el = document.getElementById('btn');
            const found = el.closest('section');
            document.getElementById('result').textContent =
              found === null ? 'null' : 'found';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "null")?;
    Ok(())
}

#[test]
fn element_closest_throws_syntax_error_for_invalid_selector() -> Result<()> {
    let html = r#"
        <div id='target'>x</div>
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('target').closest('div[');
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
