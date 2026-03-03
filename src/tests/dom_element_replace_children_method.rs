use super::*;

#[test]
fn element_replace_children_empties_node_when_called_without_arguments() -> Result<()> {
    let html = r#"
        <div id='box'><span>A</span><b>B</b></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const box = document.getElementById('box');
            const returned = box.replaceChildren();
            document.getElementById('result').textContent = [
              returned === undefined,
              box.childNodes.length,
              box.textContent === ''
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:0:true")?;
    Ok(())
}

#[test]
fn element_replace_children_replaces_with_nodes_and_text_in_order() -> Result<()> {
    let html = r#"
        <div id='source'><span>A</span><span>B</span></div>
        <div id='target'><i>X</i></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const source = document.getElementById('source');
            const target = document.getElementById('target');

            const first = source.firstElementChild;
            const second = source.lastElementChild;
            const returned = target.replaceChildren(first, 'Text', second);

            document.getElementById('result').textContent = [
              returned === undefined,
              target.childNodes.length,
              target.childNodes[0].tagName,
              target.childNodes[1].nodeName,
              target.childNodes[2].tagName,
              target.textContent,
              source.childNodes.length
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:3:SPAN:#text:SPAN:ATextB:0")?;
    Ok(())
}

#[test]
fn element_replace_children_transfers_nodes_between_elements() -> Result<()> {
    let html = r#"
        <div id='no'><span>A</span><span>B</span></div>
        <div id='yes'><span>Y</span></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const no = document.getElementById('no');
            const yes = document.getElementById('yes');

            const move1 = no.children[0];
            const move2 = no.children[1];
            const keep = yes.children[0];
            const returned = yes.replaceChildren(move1, move2, keep, '!');

            document.getElementById('result').textContent = [
              returned === undefined,
              yes.textContent,
              yes.childNodes.length,
              no.childNodes.length
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:ABY!:4:0")?;
    Ok(())
}

#[test]
fn element_replace_children_throws_hierarchy_request_error_on_invalid_tree() -> Result<()> {
    let html = r#"
        <div id='parent'><div id='child'><span id='leaf'>L</span></div></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const parent = document.getElementById('parent');
            const child = document.getElementById('child');
            let threw = false;
            try {
              child.replaceChildren(parent);
            } catch (e) {
              threw = String(e).includes('HierarchyRequestError');
            }

            document.getElementById('result').textContent = [
              threw,
              child.childNodes.length,
              child.firstElementChild.id,
              parent.contains(child)
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:1:leaf:true")?;
    Ok(())
}
