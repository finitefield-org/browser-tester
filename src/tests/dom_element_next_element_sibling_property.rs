use super::*;

#[test]
fn element_next_element_sibling_basic_example_works() -> Result<()> {
    let html = r#"
        <div id='host'>
          <div id='div-01'>Here is div-01</div>
          <div id='div-02'>Here is div-02</div>
          <script id='marker'></script>
        </div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            let el = document.getElementById('div-01').nextElementSibling;
            const names = [];
            while (el) {
              names.push(el.nodeName);
              el = el.nextElementSibling;
            }
            document.getElementById('result').textContent = names.join(',');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "DIV,SCRIPT")?;
    Ok(())
}

#[test]
fn element_next_element_sibling_ignores_non_element_nodes_and_updates_live() -> Result<()> {
    let html = r#"
        <ul id='list'><li id='a'>A</li><li id='b'>B</li><li id='c'>C</li></ul>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const list = document.getElementById('list');
            const b = document.getElementById('b');
            const c = document.getElementById('c');

            list.insertBefore(document.createTextNode('tail-text'), c);
            const before = b.nextElementSibling.id;

            c.remove();
            const after = b.nextElementSibling === null;

            document.getElementById('result').textContent = before + ':' + after;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "c:true")?;
    Ok(())
}

#[test]
fn element_next_element_sibling_property_is_read_only() -> Result<()> {
    let html = r#"
        <div id='a'>A</div><div id='b'>B</div>
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const a = document.getElementById('a');
            a.nextElementSibling = document.createElement('div');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("nextElementSibling is read-only"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}
