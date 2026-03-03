use super::*;

#[test]
fn element_prefix_returns_namespace_prefix_for_qualified_name() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const element = document.createElementNS('http://example.com/ns', 'x:div');
            document.getElementById('result').textContent =
              element.prefix + ':' + element.localName + ':' + element.tagName;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "x:div:x:div")?;
    Ok(())
}

#[test]
fn element_prefix_is_null_when_no_prefix_is_specified() -> Result<()> {
    let html = r#"
        <div id='plain'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const plain = document.getElementById('plain');
            const nsNoPrefix = document.createElementNS('http://example.com/ns', 'item');
            document.getElementById('result').textContent = [
              plain.prefix === null,
              nsNoPrefix.prefix === null
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:true")?;
    Ok(())
}

#[test]
fn element_prefix_property_is_read_only() -> Result<()> {
    let html = r#"
        <div id='plain'></div>
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const plain = document.getElementById('plain');
            plain.prefix = 'x';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("prefix is read-only"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}
