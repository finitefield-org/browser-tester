use super::*;

#[test]
fn element_local_name_returns_lowercase_name_for_html_elements() -> Result<()> {
    let html = r#"
        <div id='sample'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const element = document.getElementById('sample');
            document.getElementById('result').textContent =
              element.tagName + ':' + element.localName;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "DIV:div")?;
    Ok(())
}

#[test]
fn element_local_name_returns_local_part_of_qualified_name() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const element = document.createElement('comm:partners');
            document.getElementById('result').textContent =
              element.tagName + ':' + element.localName;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "COMM:PARTNERS:partners")?;
    Ok(())
}

#[test]
fn element_local_name_property_is_read_only() -> Result<()> {
    let html = r#"
        <div id='sample'></div>
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const element = document.getElementById('sample');
            element.localName = 'x';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("localName is read-only"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}
