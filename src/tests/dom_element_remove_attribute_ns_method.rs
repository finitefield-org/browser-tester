use super::*;

#[test]
fn element_remove_attribute_ns_removes_only_matching_namespaced_attribute() -> Result<()> {
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

            const removedNs = element.removeAttributeNS(
              'http://www.w3.org/1999/xlink',
              'href'
            );
            const afterNamespaced = [
              removedNs === undefined,
              element.getAttribute('href'),
              element.getAttributeNS('http://www.w3.org/1999/xlink', 'href') === null
            ].join('|');

            const removedPlain = element.removeAttributeNS(null, 'HREF');
            const afterPlain = [
              removedPlain === undefined,
              element.getAttribute('href') === null,
              element.getAttributeNS('http://www.w3.org/1999/xlink', 'href') === null
            ].join('|');

            document.getElementById('result').textContent = [
              afterNamespaced,
              afterPlain
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true|https://example.com|true:true|true|true")?;
    Ok(())
}

#[test]
fn element_remove_attribute_ns_treats_empty_namespace_as_null() -> Result<()> {
    let html = r#"
        <div id='box' data-count='42'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const box = document.getElementById('box');
            const removed = box.removeAttributeNS('', 'DATA-COUNT');
            document.getElementById('result').textContent = [
              removed === undefined,
              box.getAttribute('data-count') === null
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
fn element_remove_attribute_ns_is_noop_when_attribute_is_absent() -> Result<()> {
    let html = r#"
        <div id='box' data-keep='v'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const box = document.getElementById('box');
            const removed = box.removeAttributeNS('http://example.com/ns', 'missing');
            document.getElementById('result').textContent = [
              removed === undefined,
              box.getAttribute('data-keep')
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:v")?;
    Ok(())
}

#[test]
fn element_remove_attribute_ns_rejects_wrong_argument_count() -> Result<()> {
    let html = r#"
        <div id='box' data-count='42'></div>
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('box').removeAttributeNS('http://www.w3.org/1999/xlink');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("removeAttributeNS requires exactly two arguments"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}
