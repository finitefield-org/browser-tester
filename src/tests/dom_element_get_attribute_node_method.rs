use super::*;

#[test]
fn element_get_attribute_node_returns_attr_node_like_mdn_example() -> Result<()> {
    let html = r#"
        <div id='top'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const t = document.getElementById('top');
            const idAttr = t.getAttributeNode('id');
            document.getElementById('result').textContent = [
              idAttr.value === 'top',
              idAttr.name,
              idAttr.ownerElement === t
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:id:true")?;
    Ok(())
}

#[test]
fn element_get_attribute_node_lowercases_argument_and_returns_null_when_missing() -> Result<()> {
    let html = r#"
        <div id='box' data-count='42'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const box = document.getElementById('box');
            const idAttr = box.getAttributeNode('ID');
            const dataAttr = box.getAttributeNode('DATA-COUNT');
            const missing = box.getAttributeNode('missing');
            document.getElementById('result').textContent = [
              idAttr !== null ? idAttr.value : 'none',
              dataAttr !== null ? dataAttr.name : 'none',
              missing === null
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "box:data-count:true")?;
    Ok(())
}

#[test]
fn attr_node_common_tree_pointers_are_null() -> Result<()> {
    let html = r#"
        <div id='box' data-count='42'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const attr = document.getElementById('box').getAttributeNode('data-count');
            document.getElementById('result').textContent = [
              attr.parentNode === null,
              attr.previousSibling === null,
              attr.nextSibling === null
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:true:true")?;
    Ok(())
}

#[test]
fn element_get_attribute_node_rejects_missing_argument() -> Result<()> {
    let html = r#"
        <div id='box'></div>
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('box').getAttributeNode();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("getAttributeNode requires exactly one argument"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}
