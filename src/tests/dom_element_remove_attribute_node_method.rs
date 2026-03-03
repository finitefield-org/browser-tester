use super::*;

#[test]
fn element_remove_attribute_node_removes_attribute_and_returns_same_attr_object() -> Result<()> {
    let html = r#"
        <div id='box' lang='en-US' data-keep='v'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const box = document.getElementById('box');
            const langAttr = box.getAttributeNode('lang');
            const removed = box.removeAttributeNode(langAttr);
            document.getElementById('result').textContent = [
              removed === langAttr,
              removed.name,
              removed.value,
              removed.ownerElement === null,
              box.hasAttribute('lang'),
              box.getAttribute('data-keep')
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:lang:en-US:true:false:v")?;
    Ok(())
}

#[test]
fn element_remove_attribute_node_throws_not_found_for_foreign_or_detached_attr() -> Result<()> {
    let html = r#"
        <div id='first' lang='en-US'></div>
        <div id='second'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const first = document.getElementById('first');
            const second = document.getElementById('second');

            const foreignAttr = first.getAttributeNode('lang');
            let foreignThrows = false;
            try {
              second.removeAttributeNode(foreignAttr);
            } catch (e) {
              foreignThrows = String(e).includes('NotFoundError');
            }

            const detachedAttr = document.createAttribute('lang');
            let detachedThrows = false;
            try {
              second.removeAttributeNode(detachedAttr);
            } catch (e) {
              detachedThrows = String(e).includes('NotFoundError');
            }

            document.getElementById('result').textContent = [
              foreignThrows,
              detachedThrows,
              first.hasAttribute('lang'),
              second.hasAttribute('lang')
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:true:true:false")?;
    Ok(())
}

#[test]
fn element_remove_attribute_node_rejects_non_attr_argument() -> Result<()> {
    let html = r#"
        <div id='box' lang='en-US'></div>
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('box').removeAttributeNode('lang');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("removeAttributeNode argument must be an Attr"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}

#[test]
fn element_remove_attribute_node_rejects_wrong_argument_count() -> Result<()> {
    let html = r#"
        <div id='box' lang='en-US'></div>
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const attr = document.getElementById('box').getAttributeNode('lang');
            document.getElementById('box').removeAttributeNode(attr, attr);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("removeAttributeNode requires exactly one argument"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}
