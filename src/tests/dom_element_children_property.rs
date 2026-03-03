use super::*;

#[test]
fn element_children_is_live_element_only_collection_with_item_and_indexing() -> Result<()> {
    let html = r#"
        <div id='foo'><span id='a'>A</span></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const foo = document.getElementById('foo');
            const kids = foo.children;
            const sameRef = kids === foo.children;
            const before = kids.length;
            const index0 = kids[0].id;
            const item0 = kids.item(0).id;
            const itemOut = kids.item(99) === null;

            foo.appendChild(document.createTextNode('text-node'));
            const afterText = kids.length;

            const em = document.createElement('em');
            em.id = 'b';
            foo.appendChild(em);
            const afterElement = kids.length;
            const index1 = kids[1].id;
            const item1 = kids.item(1).id;

            let tags = '';
            for (const child of kids) {
              tags += child.tagName + ',';
            }

            foo.removeChild(em);
            const afterRemove = kids.length;

            document.getElementById('result').textContent = [
              sameRef,
              before,
              index0,
              item0,
              itemOut,
              afterText,
              afterElement,
              index1,
              item1,
              tags.slice(0, -1),
              afterRemove
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:1:a:a:true:1:2:b:b:SPAN,EM:1")?;
    Ok(())
}

#[test]
fn element_children_excludes_non_element_nodes_while_child_nodes_includes_them() -> Result<()> {
    let html = r#"
        <div id='host'><span id='s'></span></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const host = document.getElementById('host');
            host.appendChild(document.createTextNode('tail'));
            document.getElementById('result').textContent = [
              host.childNodes.length,
              host.children.length,
              host.children[0].id
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "2:1:s")?;
    Ok(())
}

#[test]
fn element_children_property_is_read_only() -> Result<()> {
    let html = r#"
        <div id='foo'><span id='a'>A</span></div>
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const foo = document.getElementById('foo');
            foo.children = null;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("children is read-only"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}
