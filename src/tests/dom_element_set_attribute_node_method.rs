use super::*;

#[test]
fn element_set_attribute_node_adds_new_attr_and_returns_null() -> Result<()> {
    let html = r#"
        <div id='one' lang='en-US'></div>
        <div id='two'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const source = document.getElementById('one');
            const target = document.getElementById('two');
            const sourceAttr = source.getAttributeNode('lang');

            const attr = document.createAttribute('LANG');
            attr.value = sourceAttr.value;

            const replaced = target.setAttributeNode(attr);
            document.getElementById('result').textContent = [
              replaced === null,
              target.getAttribute('lang'),
              attr.ownerElement === target
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:en-US:true")?;
    Ok(())
}

#[test]
fn element_set_attribute_node_replaces_existing_attr_and_returns_previous_attr() -> Result<()> {
    let html = r#"
        <div id='target' lang='en-US' data-keep='v'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const target = document.getElementById('target');
            const attr = document.createAttribute('LANG');
            attr.value = 'fr-FR';

            const replaced = target.setAttributeNode(attr);
            document.getElementById('result').textContent = [
              target.getAttribute('lang'),
              replaced !== null,
              replaced ? replaced.name : 'none',
              replaced ? replaced.value : 'none',
              replaced ? replaced.ownerElement === null : false,
              attr.ownerElement === target,
              target.getAttribute('data-keep')
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "fr-FR:true:lang:en-US:true:true:v")?;
    Ok(())
}

#[test]
fn element_set_attribute_node_rejects_non_attr_argument() -> Result<()> {
    let html = r#"
        <div id='target'></div>
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('target').setAttributeNode({ name: 'lang', value: 'en-US' });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("setAttributeNode argument must be an Attr"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}

#[test]
fn element_set_attribute_node_rejects_wrong_argument_count() -> Result<()> {
    let html = r#"
        <div id='target'></div>
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const attr = document.createAttribute('lang');
            document.getElementById('target').setAttributeNode(attr, attr);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("setAttributeNode requires exactly one argument"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}
