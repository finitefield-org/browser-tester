use super::*;

#[test]
fn element_get_attribute_ns_returns_string_value_for_matching_namespace() -> Result<()> {
    let html = r#"
        <div id='host'></div>
        <button id='run'>run</button>
        <pre id='result'></pre>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const host = document.getElementById('host');
            host.innerHTML =
              "<a xmlns:xlink='http://www.w3.org/1999/xlink' href='https://example.com' xlink:href='https://example.com/ns'></a>";
            const element = host.firstElementChild;

            const namespaced = element.getAttributeNS(
              'http://www.w3.org/1999/xlink',
              'href'
            );
            const plain = element.getAttributeNS(null, 'href');
            const missing = element.getAttributeNS('http://example.com/ns', 'href');

            document.getElementById('result').textContent = [
              namespaced,
              plain,
              missing === null
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "https://example.com/ns|https://example.com|true")?;
    Ok(())
}

#[test]
fn element_get_attribute_ns_treats_empty_namespace_as_null() -> Result<()> {
    let html = r#"
        <div id='box' data-count='42'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const box = document.getElementById('box');
            const value = box.getAttributeNS('', 'DATA-COUNT');
            document.getElementById('result').textContent = value;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "42")?;
    Ok(())
}

#[test]
fn element_get_attribute_ns_rejects_wrong_argument_count() -> Result<()> {
    let html = r#"
        <div id='box' data-count='42'></div>
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('box').getAttributeNS('http://www.w3.org/1999/xlink');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("getAttributeNS requires exactly two arguments"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}
