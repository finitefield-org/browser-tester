use super::*;

#[test]
fn element_last_element_child_basic_example_works() -> Result<()> {
    let html = r#"
        <ul id='list'>
          <li>First (1)</li>
          <li>Second (2)</li>
          <li>Third (3)</li>
        </ul>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const list = document.getElementById('list');
            document.getElementById('result').textContent =
              list.lastElementChild.textContent.trim();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "Third (3)")?;
    Ok(())
}

#[test]
fn element_last_element_child_ignores_non_element_nodes_and_updates_live() -> Result<()> {
    let html = r#"
        <ul id='list'><li id='a'>A</li><li id='b'>B</li></ul>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const list = document.getElementById('list');
            const text = document.createTextNode('tail-text');
            list.appendChild(text);

            const lastChildType = list.lastChild.nodeType;
            const lastElementId = list.lastElementChild.id;

            list.removeChild(document.getElementById('b'));
            const afterRemoveId = list.lastElementChild.id;

            list.removeChild(document.getElementById('a'));
            const afterEmpty = list.lastElementChild === null;

            document.getElementById('result').textContent = [
              lastChildType,
              lastElementId,
              afterRemoveId,
              afterEmpty
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "3:b:a:true")?;
    Ok(())
}

#[test]
fn element_last_element_child_property_is_read_only() -> Result<()> {
    let html = r#"
        <ul id='list'><li id='a'>A</li></ul>
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const list = document.getElementById('list');
            list.lastElementChild = document.createElement('li');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("lastElementChild is read-only"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}
