use super::*;

#[test]
fn element_get_attribute_node_ns_returns_namespaced_attr_node() -> Result<()> {
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

            const namespaced = element.getAttributeNodeNS(
              'http://www.w3.org/1999/xlink',
              'href'
            );
            const plain = element.getAttributeNodeNS(null, 'href');
            const missing = element.getAttributeNodeNS('http://example.com/ns', 'href');

            document.getElementById('result').textContent = [
              namespaced !== null ? namespaced.name : 'null',
              namespaced !== null ? namespaced.value : 'null',
              plain !== null ? plain.name : 'null',
              missing === null
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "xlink:href|https://example.com/ns|href|true")?;
    Ok(())
}

#[test]
fn element_get_attribute_node_ns_treats_empty_namespace_as_null() -> Result<()> {
    let html = r#"
        <div id='box' data-count='42'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const box = document.getElementById('box');
            const attr = box.getAttributeNodeNS('', 'DATA-COUNT');
            document.getElementById('result').textContent = [
              attr !== null,
              attr !== null ? attr.name : 'none',
              attr !== null ? attr.value : 'none'
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:data-count:42")?;
    Ok(())
}

#[test]
fn element_get_attribute_node_ns_rejects_wrong_argument_count() -> Result<()> {
    let html = r#"
        <div id='box' data-count='42'></div>
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('box').getAttributeNodeNS('http://www.w3.org/1999/xlink');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("getAttributeNodeNS requires exactly two arguments"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}
