use super::*;

#[test]
fn element_replace_with_replaces_element_with_single_node_like_mdn_example() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const div = document.createElement('div');
            const p = document.createElement('p');
            div.appendChild(p);
            const span = document.createElement('span');

            const returned = p.replaceWith(span);
            document.getElementById('result').textContent = [
              returned === undefined,
              div.outerHTML
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:<div><span></span></div>")?;
    Ok(())
}

#[test]
fn element_replace_with_without_arguments_removes_target_node() -> Result<()> {
    let html = r#"
        <div id='root'><span id='left'></span><p id='target'></p><span id='right'></span></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const root = document.getElementById('root');
            const target = document.getElementById('target');
            const returned = target.replaceWith();
            document.getElementById('result').textContent = [
              returned === undefined,
              root.childNodes.length,
              root.firstElementChild.id,
              root.lastElementChild.id,
              document.getElementById('target') === null
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:2:left:right:true")?;
    Ok(())
}

#[test]
fn element_replace_with_accepts_multiple_nodes_and_strings() -> Result<()> {
    let html = r#"
        <div id='root'><p id='target'></p><i id='tail'></i></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const root = document.getElementById('root');
            const target = document.getElementById('target');
            const span = document.createElement('span');
            span.id = 's';

            const returned = target.replaceWith('A', span, 'B');
            document.getElementById('result').textContent = [
              returned === undefined,
              root.childNodes.length,
              root.childNodes[0].nodeName,
              root.childNodes[1].id,
              root.childNodes[2].nodeName,
              root.lastElementChild.id,
              root.textContent
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:4:#text:s:#text:tail:AB")?;
    Ok(())
}

#[test]
fn element_replace_with_is_noop_for_detached_node_and_returns_undefined() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const node = document.createElement('p');
            const span = document.createElement('span');

            const returned = node.replaceWith('foo', span);
            document.getElementById('result').textContent = [
              returned === undefined,
              node.outerHTML,
              span.parentNode === null
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:<p></p>:true")?;
    Ok(())
}

#[test]
fn element_replace_with_throws_hierarchy_request_error_for_invalid_insertion() -> Result<()> {
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
              child.replaceWith(parent);
            } catch (e) {
              threw = String(e).includes('HierarchyRequestError');
            }

            document.getElementById('result').textContent = [
              threw,
              parent.contains(child),
              child.parentNode === parent
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:true:true")?;
    Ok(())
}
