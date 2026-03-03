use super::*;

#[test]
fn element_tag_name_basic_html_example_returns_uppercase() -> Result<()> {
    let html = r#"
        <span id='born'>When I was born...</span>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const span = document.getElementById('born');
            document.getElementById('result').textContent =
              span.tagName + ':' + span.nodeName + ':' + span.localName;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "SPAN:SPAN:span")?;
    Ok(())
}

#[test]
fn element_tag_name_preserves_original_case_for_non_html_namespace() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const xml = document.createElementNS('http://example.com/xml', 'SomeTag');
            const prefixed = document.createElementNS('http://example.com/xml', 'x:MixedCase');
            document.getElementById('result').textContent = [
              xml.tagName,
              xml.nodeName,
              prefixed.tagName,
              prefixed.nodeName
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "SomeTag:SomeTag:x:MixedCase:x:MixedCase")?;
    Ok(())
}

#[test]
fn element_tag_name_property_is_read_only() -> Result<()> {
    let html = r#"
        <div id='sample'></div>
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const element = document.getElementById('sample');
            element.tagName = 'section';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("tagName is read-only"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}
