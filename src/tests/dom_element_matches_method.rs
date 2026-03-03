use super::*;

#[test]
fn element_matches_mdn_example_detects_matching_selector() -> Result<()> {
    let html = r#"
        <ul id='birds'>
          <li>Orange-winged parrot</li>
          <li class='endangered'>Philippine eagle</li>
          <li>Great white pelican</li>
        </ul>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const birds = document.querySelectorAll('li');
            let endangered = '';
            for (const bird of birds) {
              if (bird.matches('.endangered')) {
                endangered = bird.textContent;
              }
            }
            document.getElementById('result').textContent = endangered;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "Philippine eagle")?;
    Ok(())
}

#[test]
fn element_matches_throws_syntax_error_for_invalid_literal_selector() -> Result<()> {
    let html = r#"
        <div id='target'></div>
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('target').matches('div[');
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

#[test]
fn element_matches_throws_syntax_error_for_invalid_dynamic_selector() -> Result<()> {
    let html = r#"
        <div id='target'></div>
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const selector = 'div[';
            document.getElementById('target').matches(selector);
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
