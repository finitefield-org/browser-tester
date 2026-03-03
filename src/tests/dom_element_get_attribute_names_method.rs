use super::*;

#[test]
fn element_get_attribute_names_returns_empty_array_when_no_attributes() -> Result<()> {
    let html = r#"
        <div id='host'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const element = document.createElement('a');
            const names = element.getAttributeNames();
            document.getElementById('result').textContent = [
              Array.isArray(names),
              names.length
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:0")?;
    Ok(())
}

#[test]
fn element_get_attribute_names_returns_qualified_names_and_plain_names() -> Result<()> {
    let html = r#"
        <div id='host'></div>
        <button id='run'>run</button>
        <pre id='result'></pre>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const host = document.getElementById('host');
            host.innerHTML = "<a href='https://example.com' xlink:href='https://example.com/x' show='new'></a>";
            const element = host.firstElementChild;

            const pairs = element
              .getAttributeNames()
              .sort()
              .map((name) => `${name}:${element.getAttribute(name)}`)
              .join('|');

            document.getElementById('result').textContent = pairs;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "href:https://example.com|show:new|xlink:href:https://example.com/x",
    )?;
    Ok(())
}

#[test]
fn element_get_attribute_names_rejects_arguments() -> Result<()> {
    let html = r#"
        <div id='host' data-x='1'></div>
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const host = document.getElementById('host');
            host.getAttributeNames('extra');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("getAttributeNames takes no arguments"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}
