use super::*;

#[test]
fn element_has_attribute_ns_returns_true_only_for_matching_namespace_and_name() -> Result<()> {
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

            const namespaced = element.hasAttributeNS(
              'http://www.w3.org/1999/xlink',
              'href'
            );
            const plain = element.hasAttributeNS(null, 'href');
            const wrongNs = element.hasAttributeNS('http://example.com/ns', 'href');
            const upperLocalName = element.hasAttributeNS(
              'http://www.w3.org/1999/xlink',
              'HREF'
            );

            document.getElementById('result').textContent = [
              namespaced,
              plain,
              wrongNs,
              upperLocalName
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true|true|false|true")?;
    Ok(())
}

#[test]
fn element_has_attribute_ns_treats_empty_namespace_as_null() -> Result<()> {
    let html = r#"
        <div id='box' data-count='42'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const box = document.getElementById('box');
            const withEmptyNamespace = box.hasAttributeNS('', 'DATA-COUNT');
            const withNullNamespace = box.hasAttributeNS(null, 'data-count');
            const missing = box.hasAttributeNS('', 'missing');

            document.getElementById('result').textContent = [
              withEmptyNamespace,
              withNullNamespace,
              missing
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:true:false")?;
    Ok(())
}

#[test]
fn element_has_attribute_ns_rejects_wrong_argument_count() -> Result<()> {
    let html = r#"
        <div id='box' data-count='42'></div>
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('box').hasAttributeNS('http://www.w3.org/1999/xlink');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("hasAttributeNS requires exactly two arguments"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}
