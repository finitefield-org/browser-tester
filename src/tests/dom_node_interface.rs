use super::*;

#[test]
fn node_remove_all_children_example_works() -> Result<()> {
    let html = r#"
        <div id='root'><span>A</span><b>B</b></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          function removeAllChildren(element) {
            while (element.firstChild) {
              element.removeChild(element.firstChild);
            }
          }

          document.getElementById('btn').addEventListener('click', () => {
            const root = document.getElementById('root');
            removeAllChildren(root);
            document.getElementById('result').textContent =
              root.childNodes.length + ':' + root.textContent;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "0:")?;
    Ok(())
}

#[test]
fn node_remove_child_simple_example_returns_removed_node() -> Result<()> {
    let html = r#"
        <div id='parent'><div id='child'></div></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const parent = document.getElementById('parent');
            const child = document.getElementById('child');
            const throwawayNode = parent.removeChild(child);

            document.getElementById('result').textContent = [
              throwawayNode === child,
              child.parentNode === null,
              parent.childNodes.length,
              document.querySelectorAll('#child').length
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:true:0:0")?;
    Ok(())
}

#[test]
fn node_remove_child_without_explicit_parent_reference_works() -> Result<()> {
    let html = r#"
        <div id='parent'><div id='child'></div></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const parent = document.getElementById('parent');
            const node = document.getElementById('child');
            if (node.parentNode) {
              node.parentNode.removeChild(node);
            }
            document.getElementById('result').textContent = [
              parent.childNodes.length,
              document.querySelectorAll('#child').length
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "0:0")?;
    Ok(())
}

#[test]
fn node_remove_child_removed_node_can_be_reused_with_listener() -> Result<()> {
    let html = r#"
        <div id='parent'><button id='child'>hit</button></div>
        <button id='btn'>run</button>
        <p id='result'>0</p>
        <script>
          const child = document.getElementById('child');
          child.addEventListener('click', () => {
            const result = document.getElementById('result');
            result.textContent = String(Number(result.textContent) + 1);
          });

          document.getElementById('btn').addEventListener('click', () => {
            const parent = document.getElementById('parent');
            const removed = parent.removeChild(child);
            parent.appendChild(removed);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.click("#child")?;
    h.assert_text("#result", "1")?;
    Ok(())
}

#[test]
fn node_remove_child_throws_type_error_like_error_for_null() {
    let err = Harness::from_html(
        r#"
        <div id='parent'></div>
        <script>
          const parentNodeEl = document.getElementById('parent');
          const garbage = parentNodeEl.removeChild(null);
        </script>
        "#,
    )
    .expect_err("removeChild should fail for null child");

    match err {
        Error::ScriptRuntime(msg) => assert_eq!(msg, "removeChild argument must be a Node"),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn node_remove_child_throws_not_found_error_like_error_when_not_child() {
    let err = Harness::from_html(
        r#"
        <div id='parent'><div id='child'></div></div>
        <script>
          const parentNodeEl = document.getElementById('parent');
          const child = document.getElementById('child');
          parentNodeEl.removeChild(child);
          document.getElementById('parent').removeChild(child);
        </script>
        "#,
    )
    .expect_err("second removeChild should fail");

    match err {
        Error::ScriptRuntime(msg) => assert_eq!(msg, "removeChild target is not a direct child"),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn node_append_child_append_paragraph_to_body_returns_new_node() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const p = document.createElement('p');
            p.id = 'newP';
            const appended = document.body.appendChild(p);

            document.getElementById('result').textContent = [
              appended === p,
              p.parentNode === document.body,
              document.querySelectorAll('#newP').length
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:true:1")?;
    Ok(())
}

#[test]
fn node_append_child_moves_existing_node_between_parents() -> Result<()> {
    let html = r#"
        <div id='left'><span id='x'>X</span></div>
        <div id='right'>R</div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const left = document.getElementById('left');
            const right = document.getElementById('right');
            const x = document.getElementById('x');
            const moved = right.appendChild(x);

            document.getElementById('result').textContent = [
              moved === x,
              left.childNodes.length,
              right.lastChild === x,
              x.parentNode === right,
              right.textContent
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:0:true:true:RX")?;
    Ok(())
}

#[test]
fn node_append_child_document_fragment_moves_children_and_returns_emptied_fragment() -> Result<()> {
    let html = r#"
        <div id='root'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const template = document.createElement('template');
            const fragment = template.content;

            const a = document.createElement('span');
            a.id = 'a';
            a.textContent = 'A';
            const b = document.createElement('span');
            b.id = 'b';
            b.textContent = 'B';

            fragment.appendChild(a);
            fragment.appendChild(b);

            const root = document.getElementById('root');
            const returned = root.appendChild(fragment);

            document.getElementById('result').textContent = [
              returned === fragment,
              fragment.childNodes.length,
              root.childNodes.length,
              root.firstChild.id,
              root.lastChild.id,
              a.parentNode === root && b.parentNode === root
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:0:2:a:b:true")?;
    Ok(())
}

#[test]
fn node_append_child_chaining_builds_nested_dom_structure() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const template = document.createElement('template');
            const fragment = template.content;
            const li = fragment
              .appendChild(document.createElement('section'))
              .appendChild(document.createElement('ul'))
              .appendChild(document.createElement('li'));
            li.textContent = 'hello world';

            document.body.appendChild(fragment);

            const section = document.body.querySelector('section');
            const ul = section.firstChild;
            const liNode = ul.firstChild;

            document.getElementById('result').textContent = [
              section.nodeName,
              ul.nodeName,
              liNode.nodeName,
              liNode.textContent,
              section.parentNode === document.body
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "SECTION:UL:LI:hello world:true")?;
    Ok(())
}

#[test]
fn node_append_child_throws_type_error_like_error_for_null() {
    let err = Harness::from_html(
        r#"
        <div id='parent'></div>
        <script>
          const parentNodeEl = document.getElementById('parent');
          const garbage = parentNodeEl.appendChild(null);
        </script>
        "#,
    )
    .expect_err("appendChild should fail for null child");

    match err {
        Error::ScriptRuntime(msg) => assert_eq!(msg, "appendChild argument must be a Node"),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn node_append_child_throws_on_cycle() {
    let err = Harness::from_html(
        r#"
        <div id='parent'><span id='child'></span></div>
        <script>
          const parentNodeEl = document.getElementById('parent');
          const child = document.getElementById('child');
          child.appendChild(parentNodeEl);
        </script>
        "#,
    )
    .expect_err("appendChild should reject a cycle");

    match err {
        Error::ScriptRuntime(msg) => assert_eq!(msg, "appendChild would create a cycle"),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn node_insert_before_example_inserts_before_reference_and_returns_node() -> Result<()> {
    let html = r#"
        <div id='parentElement'><span id='childElement'>foo bar</span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const newNode = document.createElement('span');
            newNode.id = 'newNode';
            const sp2 = document.getElementById('childElement');
            const parentDiv = sp2.parentNode;
            const inserted = parentDiv.insertBefore(newNode, sp2);

            document.getElementById('result').textContent = [
              inserted === newNode,
              parentDiv.firstChild.id,
              parentDiv.lastChild.id,
              parentDiv.childNodes.length
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:newNode:childElement:2")?;
    Ok(())
}

#[test]
fn node_insert_before_reference_null_appends_to_end_with_next_sibling() -> Result<()> {
    let html = r#"
        <div id='root'><span id='a'>A</span><span id='b'>B</span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const root = document.getElementById('root');
            const sp2 = document.getElementById('b');
            const sp1 = document.createElement('span');
            sp1.id = 'c';
            sp1.textContent = 'C';
            const inserted = root.insertBefore(sp1, sp2.nextSibling);

            document.getElementById('result').textContent = [
              inserted === sp1,
              root.textContent,
              sp1.previousSibling === sp2,
              root.lastChild.id
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:ABC:true:c")?;
    Ok(())
}

#[test]
fn node_insert_before_reference_undefined_appends_like_null() -> Result<()> {
    let html = r#"
        <div id='root'><span>A</span><span>B</span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const root = document.getElementById('root');
            const newNode = document.createElement('span');
            newNode.id = 'c';
            newNode.textContent = 'C';
            const sp2 = undefined;
            const inserted = root.insertBefore(newNode, sp2);
            document.getElementById('result').textContent = [
              inserted === newNode,
              root.textContent,
              root.lastChild.id
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:ABC:c")?;
    Ok(())
}

#[test]
fn node_insert_before_moves_existing_node_between_parents() -> Result<()> {
    let html = r#"
        <div id='left'><span id='x'>X</span></div>
        <div id='right'><span id='y'>Y</span><span id='z'>Z</span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const left = document.getElementById('left');
            const right = document.getElementById('right');
            const x = document.getElementById('x');
            const z = document.getElementById('z');
            const inserted = right.insertBefore(x, z);

            document.getElementById('result').textContent = [
              inserted === x,
              left.childNodes.length,
              right.textContent,
              x.parentNode === right,
              right.childNodes[1].id
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:0:YXZ:true:x")?;
    Ok(())
}

#[test]
fn node_insert_before_document_fragment_moves_children_and_returns_fragment() -> Result<()> {
    let html = r#"
        <div id='root'><span id='tail'>T</span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const template = document.createElement('template');
            const fragment = template.content;
            const a = document.createElement('span');
            a.textContent = 'A';
            const b = document.createElement('span');
            b.textContent = 'B';
            fragment.appendChild(a);
            fragment.appendChild(b);

            const root = document.getElementById('root');
            const tail = document.getElementById('tail');
            const inserted = root.insertBefore(fragment, tail);

            document.getElementById('result').textContent = [
              inserted === fragment,
              fragment.childNodes.length,
              root.textContent,
              root.firstChild.textContent,
              root.childNodes[1].textContent,
              a.parentNode === root && b.parentNode === root
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:0:ABT:A:B:true")?;
    Ok(())
}

#[test]
fn node_insert_before_throws_when_reference_argument_is_missing() {
    let err = Harness::from_html(
        r#"
        <script>
          const root = document.createElement('div');
          const newNode = document.createElement('span');
          const inserted = root.insertBefore(newNode);
        </script>
        "#,
    )
    .expect_err("insertBefore should require reference argument");

    match err {
        Error::ScriptRuntime(msg) => assert_eq!(msg, "insertBefore requires exactly two arguments"),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn node_insert_before_throws_for_invalid_reference_argument_type() {
    let err = Harness::from_html(
        r#"
        <script>
          const root = document.createElement('div');
          const newNode = document.createElement('span');
          const inserted = root.insertBefore(newNode, 'undefined');
        </script>
        "#,
    )
    .expect_err("insertBefore should reject non-node, non-null reference");

    match err {
        Error::ScriptRuntime(msg) => {
            assert_eq!(msg, "insertBefore second argument must be a Node or null")
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn node_insert_before_throws_when_reference_is_not_direct_child() {
    let err = Harness::from_html(
        r#"
        <div id='root'><span id='a'>A</span></div>
        <script>
          const root = document.getElementById('root');
          const newNode = document.createElement('span');
          const detached = document.createElement('span');
          const inserted = root.insertBefore(newNode, detached);
        </script>
        "#,
    )
    .expect_err("insertBefore should reject non-child reference");

    match err {
        Error::ScriptRuntime(msg) => {
            assert_eq!(msg, "insertBefore reference is not a direct child")
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn node_insert_before_throws_on_cycle() {
    let err = Harness::from_html(
        r#"
        <div id='parent'><span id='child'><b id='grand'></b></span></div>
        <script>
          const parentNodeEl = document.getElementById('parent');
          const child = document.getElementById('child');
          const grand = document.getElementById('grand');
          const inserted = child.insertBefore(parentNodeEl, grand);
        </script>
        "#,
    )
    .expect_err("insertBefore should reject a cycle");

    match err {
        Error::ScriptRuntime(msg) => assert_eq!(msg, "insertBefore would create a cycle"),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn node_clone_node_example_deep_true_copies_subtree() -> Result<()> {
    let html = r#"
        <div id='wrap'><p id='para1' class='alpha'>Hello<span>World</span></p></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const p = document.getElementById('para1');
            const p2 = p.cloneNode(true);
            document.getElementById('wrap').appendChild(p2);

            document.getElementById('result').textContent = [
              p2 !== p,
              p2.nodeName,
              p2.getAttribute('class'),
              p2.childNodes.length,
              p2.textContent,
              p2.parentNode.id
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:P:alpha:2:HelloWorld:wrap")?;
    Ok(())
}

#[test]
fn node_clone_node_shallow_clone_has_no_children_and_starts_detached() -> Result<()> {
    let html = r#"
        <div id='root'><p id='src' data-x='1'>A<span>B</span></p></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const src = document.getElementById('src');
            const clone = src.cloneNode();
            const detached = clone.parentNode === null;
            document.getElementById('root').appendChild(clone);

            document.getElementById('result').textContent = [
              clone !== src,
              detached,
              clone.getAttribute('data-x'),
              clone.childNodes.length,
              clone.textContent,
              clone.parentNode.id,
              clone.previousSibling === src
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:true:1:0::root:true")?;
    Ok(())
}

#[test]
fn node_clone_node_text_node_roundtrips_value_and_is_detached() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const text = document.createTextNode('alpha');
            const clone = text.cloneNode(true);

            document.getElementById('result').textContent = [
              clone !== text,
              clone.nodeType === Node.TEXT_NODE,
              clone.nodeValue,
              clone.parentNode === null
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:true:alpha:true")?;
    Ok(())
}

#[test]
fn node_clone_node_does_not_copy_add_event_listener_handlers() -> Result<()> {
    let html = r#"
        <div id='mount'></div>
        <p id='result'>0</p>
        <script>
          const mount = document.getElementById('mount');
          const result = document.getElementById('result');
          let count = 0;

          const original = document.createElement('button');
          original.id = 'original';
          original.textContent = 'original';
          original.addEventListener('click', () => {
            count++;
            result.textContent = String(count);
          });

          const clone = original.cloneNode(true);
          clone.id = 'clone';

          mount.appendChild(original);
          mount.appendChild(clone);
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#original")?;
    h.click("#clone")?;
    h.assert_text("#result", "1")?;
    Ok(())
}

#[test]
fn node_clone_node_document_fragment_deep_true_can_be_appended() -> Result<()> {
    let html = r#"
        <div id='host'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const template = document.createElement('template');
            template.innerHTML = '<section><p>hello</p></section>';
            const fragment = template.content;
            const clone = fragment.cloneNode(true);
            const host = document.getElementById('host');
            const returned = host.appendChild(clone);

            document.getElementById('result').textContent = [
              returned === clone,
              clone.nodeType === Node.DOCUMENT_FRAGMENT_NODE,
              clone.childNodes.length,
              host.querySelectorAll('section').length,
              host.textContent
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:true:0:1:hello")?;
    Ok(())
}

#[test]
fn node_clone_node_throws_when_called_with_too_many_arguments() {
    let err = Harness::from_html(
        r#"
        <script>
          const root = document.createElement('div');
          const cloned = root.cloneNode(true, false);
        </script>
        "#,
    )
    .expect_err("cloneNode should reject more than one argument");

    match err {
        Error::ScriptRuntime(msg) => assert_eq!(msg, "cloneNode supports at most one argument"),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn node_type_distinguishes_document_element_text_and_fragment() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const p = document.createElement('p');
            p.textContent = 'Once upon a time';

            const template = document.createElement('template');
            template.innerHTML = '<span>X</span>';
            const fragment = template.content;

            const before = p.nodeType;
            p.nodeType = 99;

            document.getElementById('result').textContent = [
              document.nodeType === Node.DOCUMENT_NODE,
              fragment.nodeType === Node.DOCUMENT_FRAGMENT_NODE,
              p.nodeType === Node.ELEMENT_NODE,
              p.firstChild.nodeType === Node.TEXT_NODE,
              p.nodeType === before,
              Node.ELEMENT_NODE,
              Node.ATTRIBUTE_NODE,
              Node.TEXT_NODE,
              Node.CDATA_SECTION_NODE,
              Node.PROCESSING_INSTRUCTION_NODE,
              Node.COMMENT_NODE,
              Node.DOCUMENT_NODE,
              Node.DOCUMENT_TYPE_NODE,
              Node.DOCUMENT_FRAGMENT_NODE
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:true:true:true:true:1:2:3:4:7:8:9:10:11")?;
    Ok(())
}

#[test]
fn node_type_comment_example_condition_is_true_without_comment_node() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const root = document.documentElement;
            const node = root.firstChild;
            const shouldWarn = !node || node.nodeType !== Node.COMMENT_NODE;
            document.getElementById('result').textContent = String(shouldWarn);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true")?;
    Ok(())
}

#[test]
fn node_text_content_get_and_set_example_works() -> Result<()> {
    let html = r#"
        <div id='divA'>This is <span>some</span> text!</div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const div = document.getElementById('divA');
            const before = div.textContent;

            div.textContent = 'This text is different!';

            document.getElementById('result').textContent = [
              before,
              div.textContent,
              div.innerHTML,
              div.childNodes.length,
              div.firstChild.nodeType === Node.TEXT_NODE
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "This is some text!|This text is different!|This text is different!|1|true",
    )?;
    Ok(())
}

#[test]
fn node_text_content_document_is_null_and_document_element_contains_text() -> Result<()> {
    let html = r#"
        <p>Alpha</p>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const d = document;
            const before = d.textContent === null;
            const allTextBefore = d.documentElement.textContent.includes('Alpha');
            d.textContent = 'ignored';
            const allTextAfter = d.documentElement.textContent.includes('Alpha');
            const after = d.textContent === null;

            document.getElementById('result').textContent = [
              before,
              allTextBefore,
              allTextAfter,
              after
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:true:true:true")?;
    Ok(())
}

#[test]
fn node_text_content_on_text_node_roundtrips_with_node_value() -> Result<()> {
    let html = r#"
        <div id='root'>A<span>B</span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const root = document.getElementById('root');
            const text = root.firstChild;

            const before = text.textContent;
            text.textContent = 'Z';

            document.getElementById('result').textContent = [
              before,
              text.textContent,
              text.nodeValue,
              root.textContent
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "A:Z:Z:ZB")?;
    Ok(())
}

#[test]
fn node_replace_child_mdn_example_works() -> Result<()> {
    let html = r#"
        <div id='parent'><span id='childSpan'>foo bar</span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const sp1 = document.createElement('span');
            sp1.id = 'newSpan';
            const sp1Content = document.createTextNode('new replacement span element.');
            sp1.appendChild(sp1Content);

            const sp2 = document.getElementById('childSpan');
            const parentDiv = sp2.parentNode;
            const replaced = parentDiv.replaceChild(sp1, sp2);

            document.getElementById('result').textContent = [
              replaced.id,
              parentDiv.querySelector('#newSpan').textContent,
              parentDiv.textContent,
              parentDiv.querySelectorAll('#childSpan').length
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "childSpan:new replacement span element.:new replacement span element.:0",
    )?;
    Ok(())
}

#[test]
fn node_replace_child_moves_existing_node_from_other_parent() -> Result<()> {
    let html = r#"
        <div id='left'><span id='x'>X</span></div>
        <div id='right'><span id='y'>Y</span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const left = document.getElementById('left');
            const right = document.getElementById('right');
            const x = document.getElementById('x');
            const y = document.getElementById('y');
            const replaced = left.replaceChild(y, x);

            document.getElementById('result').textContent = [
              replaced.id,
              '[' + left.textContent + ']',
              '[' + right.textContent + ']',
              y.parentNode === left,
              x.parentNode === null
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "x:[Y]:[]:true:true")?;
    Ok(())
}

#[test]
fn node_replace_child_with_same_node_is_noop_for_direct_child() -> Result<()> {
    let html = r#"
        <div id='root'><span id='a'>A</span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const root = document.getElementById('root');
            const a = document.getElementById('a');
            const replaced = root.replaceChild(a, a);

            document.getElementById('result').textContent = [
              replaced === a,
              root.childNodes.length,
              root.firstChild.id
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:1:a")?;
    Ok(())
}

#[test]
fn node_replace_child_throws_when_old_child_is_not_direct_child() {
    let err = Harness::from_html(
        r#"
        <div id='root'><span id='old'>old</span></div>
        <script>
          const root = document.getElementById('root');
          const detached = document.createElement('span');
          root.replaceChild(detached, detached);
        </script>
        "#,
    )
    .expect_err("replaceChild should reject when oldChild is not a direct child");

    match err {
        Error::ScriptRuntime(msg) => assert_eq!(msg, "replaceChild target is not a direct child"),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn node_replace_child_throws_on_cycle() {
    let err = Harness::from_html(
        r#"
        <div id='parent'><span id='child'><b id='grand'></b></span></div>
        <script>
          const parentNodeEl = document.getElementById('parent');
          const child = document.getElementById('child');
          const grand = document.getElementById('grand');
          child.replaceChild(parentNodeEl, grand);
        </script>
        "#,
    )
    .expect_err("replaceChild should reject a cycle");

    match err {
        Error::ScriptRuntime(msg) => assert_eq!(msg, "replaceChild would create a cycle"),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn node_child_nodes_is_live_and_returns_same_object() -> Result<()> {
    let html = r#"
        <div id='root'><span>A</span>T</div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const root = document.getElementById('root');
            const children = root.childNodes;
            const sameObject = children === root.childNodes;
            const firstIsElement = children[0].nodeName === 'SPAN';
            const secondIsText = children[1].nodeType === Node.TEXT_NODE;
            const before = children.length;

            root.appendChild(document.createElement('b'));
            const afterAppend = children.length;

            root.removeChild(root.firstChild);
            const afterRemove = children.length;
            const firstNameAfterRemove = children[0].nodeName;

            document.getElementById('result').textContent = [
              sameObject,
              firstIsElement,
              secondIsText,
              before,
              afterAppend,
              afterRemove,
              firstNameAfterRemove
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:true:true:2:3:2:#text")?;
    Ok(())
}

#[test]
fn node_child_nodes_simple_usage_for_of_works() -> Result<()> {
    let html = r#"
        <p id='para'><span>A</span><span>B</span></p>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const para = document.getElementById('para');
            const names = [];
            if (para.hasChildNodes()) {
              const children = para.childNodes;
              for (const node of children) {
                names.push(node.nodeName);
              }
            }
            document.getElementById('result').textContent = names.join(',');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "SPAN,SPAN")?;
    Ok(())
}

#[test]
fn node_each_node_and_grep_example_works() -> Result<()> {
    let html = r#"
        <div id='root'>A<span>B</span><p>C</p></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          function eachNode(rootNode, callback) {
            if (!callback) {
              const nodes = [];
              eachNode(rootNode, (node) => {
                nodes.push(node);
              });
              return nodes;
            }

            if (callback(rootNode) === false) {
              return false;
            }

            if (rootNode.hasChildNodes()) {
              for (const node of rootNode.childNodes) {
                if (eachNode(node, callback) === false) {
                  return;
                }
              }
            }
          }

          function grep(parentNode, pattern) {
            const matches = [];

            eachNode(parentNode, (node) => {
              if (node.nodeType !== Node.TEXT_NODE) {
                return;
              }
              if (node.textContent.includes(pattern)) {
                matches.push(node);
              }
            });

            return matches;
          }

          document.getElementById('btn').addEventListener('click', () => {
            const root = document.getElementById('root');
            const nodes = eachNode(root);
            const matches = grep(root, 'B');
            document.getElementById('result').textContent =
              nodes.length + ':' + matches.length + ':' + matches[0].textContent + ':' +
              (nodes[1].nodeType === Node.TEXT_NODE);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "6:1:B:true")?;
    Ok(())
}

#[test]
fn node_core_properties_and_tree_methods_work() -> Result<()> {
    let html = r#"
        <div id='root'>A<span id='b'>B</span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const root = document.getElementById('root');
            const text = root.firstChild;
            const span = document.getElementById('b');

            const nodeProps = [
              Node.ATTRIBUTE_NODE,
              Node.CDATA_SECTION_NODE,
              Node.PROCESSING_INSTRUCTION_NODE,
              Node.DOCUMENT_TYPE_NODE,
              root.nodeName,
              text.nodeName,
              text.nodeValue,
              text.parentNode === root,
              text.parentElement === root,
              text.nextSibling === span,
              span.previousSibling === text,
              root.ownerDocument.nodeType === Node.DOCUMENT_NODE,
              root.isConnected,
              root.hasChildNodes()
            ].join(',');

            const c = document.createElement('i');
            c.textContent = 'C';
            const appendRet = root.appendChild(c) === c;

            const d = document.createElement('u');
            d.textContent = 'D';
            const insertRet = root.insertBefore(d, c) === d;

            const replacement = document.createElement('em');
            replacement.textContent = 'R';
            const old = root.replaceChild(replacement, span);

            const removeRet = root.removeChild(d) === d;

            const methods = [
              appendRet,
              insertRet,
              old === span,
              removeRet,
              root.contains(replacement),
              root.contains(root),
              root.getRootNode().nodeType === Node.DOCUMENT_NODE,
              replacement.isSameNode(replacement),
              replacement.isEqualNode(replacement.cloneNode(true)),
              root.textContent
            ].join(',');

            document.getElementById('result').textContent = nodeProps + '|' + methods;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "2,4,7,10,DIV,#text,A,true,true,true,true,true,true,true|true,true,true,true,true,true,true,true,true,ARC",
    )?;
    Ok(())
}

#[test]
fn node_is_connected_standard_dom_example_transitions_correctly() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const test = document.createElement('p');
            const before = test.isConnected;

            document.body.appendChild(test);
            const afterAppend = test.isConnected;

            document.body.removeChild(test);
            const afterRemove = test.isConnected;

            document.getElementById('result').textContent = [
              before,
              afterAppend,
              afterRemove
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "false:true:false")?;
    Ok(())
}

#[test]
fn node_is_connected_is_true_for_document_and_attached_nodes() -> Result<()> {
    let html = r#"
        <div id='root'>A<span id='child'>B</span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const body = document.body;
            const root = document.getElementById('root');
            const child = document.getElementById('child');
            const text = root.firstChild;

            document.getElementById('result').textContent = [
              body.isConnected,
              root.isConnected,
              child.isConnected,
              text.isConnected
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:true:true:true")?;
    Ok(())
}

#[test]
fn node_is_connected_document_fragment_child_becomes_connected_when_inserted() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const template = document.createElement('template');
            const fragment = template.content;
            const style = document.createElement('style');

            const before = style.isConnected;
            fragment.appendChild(style);
            const inFragment = style.isConnected;

            document.body.appendChild(fragment);
            const afterInsert = style.isConnected;
            const fragmentConnected = fragment.isConnected;

            document.getElementById('result').textContent = [
              before,
              inFragment,
              afterInsert,
              fragmentConnected
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "false:false:true:false")?;
    Ok(())
}

#[test]
fn node_parent_element_usage_example_can_style_parent_element() -> Result<()> {
    let html = r#"
        <div id='parent'><span id='child'>text</span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const node = document.getElementById('child');
            if (node.parentElement) {
              node.parentElement.style.color = 'red';
            }
            const parent = document.getElementById('parent');
            document.getElementById('result').textContent = [
              parent.style.color,
              node.parentElement === parent
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "red:true")?;
    Ok(())
}

#[test]
fn node_parent_element_is_null_when_parent_is_document() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const htmlNode = document.documentElement;
            document.getElementById('result').textContent = [
              htmlNode.parentElement === null,
              htmlNode.parentNode.nodeType === Node.DOCUMENT_NODE
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:true")?;
    Ok(())
}

#[test]
fn node_parent_element_is_null_when_parent_is_document_fragment() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const template = document.createElement('template');
            const fragment = template.content;
            const child = document.createElement('span');
            fragment.appendChild(child);

            document.getElementById('result').textContent = [
              child.parentNode === fragment,
              child.parentElement === null
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:true")?;
    Ok(())
}

#[test]
fn node_parent_node_usage_example_removes_node_when_present() -> Result<()> {
    let html = r#"
        <div id='parent'><span id='node'>X</span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const node = document.getElementById('node');
            const parent = document.getElementById('parent');
            const before = node.parentNode !== null;
            if (node.parentNode) {
              node.parentNode.removeChild(node);
            }
            const after = node.parentNode === null;
            const parentChildren = parent.childNodes.length;

            document.getElementById('result').textContent = [
              before,
              after,
              parentChildren
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:true:0")?;
    Ok(())
}

#[test]
fn node_parent_node_returns_expected_parent_types_and_null_cases() -> Result<()> {
    let html = r#"
        <div id='root'>A<span id='child'>B</span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const root = document.getElementById('root');
            const text = root.firstChild;
            const htmlNode = document.documentElement;
            const documentNode = htmlNode.parentNode;
            const detached = document.createElement('i');

            const template = document.createElement('template');
            const fragment = template.content;
            const fragChild = document.createElement('span');
            fragment.appendChild(fragChild);

            document.getElementById('result').textContent = [
              text.parentNode === root,
              htmlNode.parentNode.nodeType === Node.DOCUMENT_NODE,
              documentNode.parentNode === null,
              fragChild.parentNode === fragment,
              fragment.parentNode === null,
              detached.parentNode === null
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:true:true:true:true:true")?;
    Ok(())
}

#[test]
fn node_contains_returns_expected_values_for_self_descendants_and_null() -> Result<()> {
    let html = r#"
        <div id='root'><span id='child'><b id='grand'>G</b></span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const root = document.getElementById('root');
            const child = document.getElementById('child');
            const grand = document.getElementById('grand');
            const detached = document.createElement('i');

            document.getElementById('result').textContent = [
              root.contains(root),
              root.contains(child),
              root.contains(grand),
              child.contains(root),
              root.contains(null),
              root.contains(detached)
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:true:true:false:false:false")?;
    Ok(())
}

#[test]
fn node_contains_mdn_is_in_page_example_works() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          function isInPage(node) {
            return node === document.body ? false : document.body.contains(node);
          }

          document.getElementById('btn').addEventListener('click', () => {
            const inBody = document.createElement('div');
            document.body.appendChild(inBody);
            const detached = document.createElement('span');

            document.getElementById('result').textContent = [
              isInPage(document.body),
              isInPage(inBody),
              isInPage(detached)
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "false:true:false")?;
    Ok(())
}

#[test]
fn node_contains_throws_when_argument_is_missing() {
    let err = Harness::from_html(
        r#"
        <script>
          const root = document.createElement('div');
          const result = root.contains();
        </script>
        "#,
    )
    .expect_err("contains should require one argument");

    match err {
        Error::ScriptRuntime(msg) => assert_eq!(msg, "contains requires exactly one argument"),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn node_contains_throws_for_non_node_non_null_argument() {
    let err = Harness::from_html(
        r#"
        <script>
          const root = document.createElement('div');
          const result = root.contains(1);
        </script>
        "#,
    )
    .expect_err("contains should reject non-node arguments");

    match err {
        Error::ScriptRuntime(msg) => assert_eq!(msg, "contains argument must be a Node or null"),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn node_get_root_node_example_returns_document_node() -> Result<()> {
    let html = r#"
        <div id='node'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const node = document.getElementById('node');
            const rootNode = node.getRootNode();
            document.getElementById('result').textContent = [
              rootNode.nodeType === Node.DOCUMENT_NODE,
              rootNode.nodeName
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:#document")?;
    Ok(())
}

#[test]
fn node_get_root_node_options_argument_is_accepted() -> Result<()> {
    let html = r#"
        <div id='node'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const node = document.getElementById('node');
            const base = node.getRootNode();
            const composedFalse = node.getRootNode({ composed: false });
            const composedTrue = node.getRootNode({ composed: true });
            const primitiveArg = node.getRootNode(1);

            document.getElementById('result').textContent = [
              base === composedFalse,
              composedFalse === composedTrue,
              composedTrue === primitiveArg,
              composedTrue.nodeName
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:true:true:#document")?;
    Ok(())
}

#[test]
fn node_get_root_node_returns_root_of_unmounted_tree() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const element = document.createElement('p');
            const child = document.createElement('span');
            element.appendChild(child);

            const rootNode = child.getRootNode();

            document.getElementById('result').textContent = [
              element === rootNode,
              element === element.getRootNode(),
              rootNode.nodeName
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:true:P")?;
    Ok(())
}

#[test]
fn node_get_root_node_throws_when_called_with_too_many_arguments() {
    let err = Harness::from_html(
        r#"
        <script>
          const root = document.createElement('div');
          const value = root.getRootNode({}, {});
        </script>
        "#,
    )
    .expect_err("getRootNode should reject more than one argument");

    match err {
        Error::ScriptRuntime(msg) => {
            assert_eq!(msg, "getRootNode supports at most one options argument")
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn node_compare_document_position_mdn_head_before_body_example_works() -> Result<()> {
    let html = r#"
        <div id='root'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const root = document.getElementById('root');
            const head = document.createElement('div');
            const body = document.createElement('div');
            root.appendChild(head);
            root.appendChild(body);
            const wellFormed =
              (head.compareDocumentPosition(body) & Node.DOCUMENT_POSITION_FOLLOWING) !== 0;
            document.getElementById('result').textContent = String(wellFormed);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true")?;
    Ok(())
}

#[test]
fn node_compare_document_position_sets_expected_bitmasks() -> Result<()> {
    let html = r#"
        <div id='root'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const root = document.getElementById('root');
            const a = document.createElement('section');
            const b = document.createElement('span');
            const c = document.createElement('em');
            root.appendChild(a);
            a.appendChild(b);
            root.appendChild(c);

            const same = a.compareDocumentPosition(a);
            const containedByAndFollowing = a.compareDocumentPosition(b);
            const containsAndPreceding = b.compareDocumentPosition(a);
            const followingSibling = b.compareDocumentPosition(c);
            const precedingSibling = c.compareDocumentPosition(b);

            const detached = document.createElement('div');
            const bToDetached = b.compareDocumentPosition(detached);
            const detachedToB = detached.compareDocumentPosition(b);

            const disconnectedSet =
              (bToDetached & Node.DOCUMENT_POSITION_DISCONNECTED) !== 0 &&
              (detachedToB & Node.DOCUMENT_POSITION_DISCONNECTED) !== 0;
            const implSpecificSet =
              (bToDetached & Node.DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC) !== 0 &&
              (detachedToB & Node.DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC) !== 0;

            const bToDetachedPre = (bToDetached & Node.DOCUMENT_POSITION_PRECEDING) !== 0;
            const bToDetachedFol = (bToDetached & Node.DOCUMENT_POSITION_FOLLOWING) !== 0;
            const detachedToBPre = (detachedToB & Node.DOCUMENT_POSITION_PRECEDING) !== 0;
            const detachedToBFol = (detachedToB & Node.DOCUMENT_POSITION_FOLLOWING) !== 0;
            const oneDirectionEach = (bToDetachedPre !== bToDetachedFol) &&
              (detachedToBPre !== detachedToBFol);
            const oppositeDirections =
              bToDetachedPre === detachedToBFol &&
              bToDetachedFol === detachedToBPre;

            document.getElementById('result').textContent = [
              same,
              containedByAndFollowing,
              containsAndPreceding,
              followingSibling,
              precedingSibling,
              disconnectedSet,
              implSpecificSet,
              oneDirectionEach,
              oppositeDirections
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "0:20:10:4:2:true:true:true:true")?;
    Ok(())
}

#[test]
fn node_compare_document_position_throws_for_non_node_argument() {
    let err = Harness::from_html(
        r#"
        <script>
          const a = document.createElement('div');
          const pos = a.compareDocumentPosition(null);
        </script>
        "#,
    )
    .expect_err("compareDocumentPosition should fail for non-node argument");

    match err {
        Error::ScriptRuntime(msg) => {
            assert_eq!(msg, "compareDocumentPosition argument must be a Node")
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn node_is_equal_node_mdn_div_example_reports_expected_results() -> Result<()> {
    let html = r#"
        <div>This is the first element.</div>
        <div>This is the second element.</div>
        <div>This is the first element.</div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const divList = document.getElementsByTagName('div');
            document.getElementById('result').textContent = [
              divList[0].isEqualNode(divList[0]),
              divList[0].isEqualNode(divList[1]),
              divList[0].isEqualNode(divList[2])
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:false:true")?;
    Ok(())
}

#[test]
fn node_is_equal_node_compares_attributes_descendants_and_nullish_values() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const base = document.createElement('div');
            base.setAttribute('data-kind', 'sample');
            const child = document.createElement('span');
            child.textContent = 'Alpha';
            base.appendChild(child);

            const same = base.cloneNode(true);

            const differentAttr = base.cloneNode(true);
            differentAttr.setAttribute('data-kind', 'other');

            const differentChild = base.cloneNode(true);
            differentChild.firstChild.textContent = 'Beta';

            const differentType = document.createTextNode('Alpha');

            document.getElementById('result').textContent = [
              base.isEqualNode(same),
              base.isEqualNode(differentAttr),
              base.isEqualNode(differentChild),
              base.isEqualNode(differentType),
              base.isEqualNode(null),
              base.isEqualNode(undefined)
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:false:false:false:false:false")?;
    Ok(())
}

#[test]
fn node_is_equal_node_throws_for_non_node_non_null_argument() {
    let err = Harness::from_html(
        r#"
        <script>
          const node = document.createElement('div');
          node.isEqualNode('not-a-node');
        </script>
        "#,
    )
    .expect_err("isEqualNode should reject non-node, non-null argument");

    match err {
        Error::ScriptRuntime(msg) => {
            assert_eq!(msg, "isEqualNode argument must be a Node or null")
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn node_is_equal_node_throws_for_too_many_arguments() {
    let err = Harness::from_html(
        r#"
        <script>
          const node = document.createElement('div');
          node.isEqualNode(node, node);
        </script>
        "#,
    )
    .expect_err("isEqualNode should reject more than one argument");

    match err {
        Error::ScriptRuntime(msg) => assert_eq!(msg, "isEqualNode supports at most one argument"),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn node_is_same_node_mdn_div_example_reports_expected_results() -> Result<()> {
    let html = r#"
        <div>This is the first element.</div>
        <div>This is the second element.</div>
        <div>This is the first element.</div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const divList = document.getElementsByTagName('div');
            document.getElementById('result').textContent = [
              divList[0].isSameNode(divList[0]),
              divList[0].isSameNode(divList[1]),
              divList[0].isSameNode(divList[2])
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:false:false")?;
    Ok(())
}

#[test]
fn node_is_same_node_is_strict_identity_alias() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const base = document.createElement('div');
            base.textContent = 'same-content';
            const clone = base.cloneNode(true);
            const ref = base;

            document.getElementById('result').textContent = [
              base.isSameNode(ref),
              base === ref,
              base.isSameNode(clone),
              base === clone
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:true:false:false")?;
    Ok(())
}

#[test]
fn node_is_same_node_returns_false_for_nullish_and_missing_argument() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const node = document.createElement('div');
            document.getElementById('result').textContent = [
              node.isSameNode(null),
              node.isSameNode(undefined),
              node.isSameNode()
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "false:false:false")?;
    Ok(())
}

#[test]
fn node_is_same_node_throws_for_non_node_non_null_argument() {
    let err = Harness::from_html(
        r#"
        <script>
          const node = document.createElement('div');
          node.isSameNode('not-a-node');
        </script>
        "#,
    )
    .expect_err("isSameNode should reject non-node, non-null argument");

    match err {
        Error::ScriptRuntime(msg) => {
            assert_eq!(msg, "isSameNode argument must be a Node or null")
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn node_is_same_node_throws_for_too_many_arguments() {
    let err = Harness::from_html(
        r#"
        <script>
          const node = document.createElement('div');
          node.isSameNode(node, node);
        </script>
        "#,
    )
    .expect_err("isSameNode should reject more than one argument");

    match err {
        Error::ScriptRuntime(msg) => assert_eq!(msg, "isSameNode supports at most one argument"),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn node_compare_normalize_and_namespace_methods_work() -> Result<()> {
    let html = r#"
        <div id='root'><span id='a'>A</span><span id='b'>B</span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const root = document.getElementById('root');
            const a = document.getElementById('a');
            const b = document.getElementById('b');

            const ab = a.compareDocumentPosition(b);
            const ba = b.compareDocumentPosition(a);
            const positionOk =
              (ab & Node.DOCUMENT_POSITION_FOLLOWING) !== 0 &&
              (ba & Node.DOCUMENT_POSITION_PRECEDING) !== 0;

            root.appendChild(document.createTextNode(''));
            root.appendChild(document.createTextNode('X'));
            root.appendChild(document.createTextNode('Y'));
            root.appendChild(document.createTextNode(''));
            const before = root.childNodes.length;
            root.normalize();
            const after = root.childNodes.length;
            const merged = root.lastChild.nodeValue;

            const namespaceUri = root.lookupNamespaceURI(null);
            const prefixIsNull = root.lookupPrefix(namespaceUri) === null;
            const defaultNamespace = root.isDefaultNamespace(namespaceUri);

            const textNode = a.firstChild;
            textNode.nodeValue = 'Z';

            document.getElementById('result').textContent = [
              positionOk,
              before,
              after,
              merged,
              namespaceUri,
              prefixIsNull,
              defaultNamespace,
              a.textContent
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "true:6:3:XY:http://www.w3.org/1999/xhtml:true:true:Z",
    )?;
    Ok(())
}
