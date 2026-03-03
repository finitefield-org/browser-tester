use super::*;

#[test]
fn element_namespace_uri_returns_xhtml_for_html_elements() -> Result<()> {
    let html = r#"
        <div id='sample'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const element = document.getElementById('sample');
            document.getElementById('result').textContent =
              element.localName + ':' + element.namespaceURI;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "div:http://www.w3.org/1999/xhtml")?;
    Ok(())
}

#[test]
fn element_namespace_uri_from_create_element_ns_is_frozen_at_creation() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const element = document.createElementNS(
              'http://www.w3.org/2000/svg',
              'svg:circle'
            );
            element.setAttribute('xmlns', 'http://www.w3.org/1999/xhtml');
            document.body.appendChild(element);
            document.getElementById('result').textContent =
              element.localName + ':' + element.namespaceURI;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "circle:http://www.w3.org/2000/svg")?;
    Ok(())
}

#[test]
fn element_namespace_uri_can_be_null_with_create_element_ns_null_namespace() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const element = document.createElementNS(null, 'plain');
            document.getElementById('result').textContent =
              (element.namespaceURI === null) + ':' + element.localName;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:plain")?;
    Ok(())
}

#[test]
fn element_namespace_uri_property_is_read_only() -> Result<()> {
    let html = r#"
        <div id='sample'></div>
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const element = document.getElementById('sample');
            element.namespaceURI = 'http://example.com/ns';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("namespaceURI is read-only"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}
