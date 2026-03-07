use super::*;

#[test]
fn query_selector_all_index_supports_expression() -> Result<()> {
    let html = r#"
        <ul>
          <li class='item'>A</li>
          <li class='item'>B</li>
          <li class='item'>C</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const items = document.querySelectorAll('.item');
            const index = 1;
            const next = items[index + 1].textContent;
            document.getElementById('result').textContent = items[index].textContent + ':' + next;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "B:C")?;
    Ok(())
}

#[test]
fn query_selector_all_list_index_after_reuse_works() -> Result<()> {
    let html = r#"
        <ul>
          <li class='item'>A</li>
          <li class='item'>B</li>
          <li class='item'>C</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const items = document.querySelectorAll('.item');
            const picked = items[2];
            document.getElementById('result').textContent = picked.textContent + ':' + items.length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "C:3")?;
    Ok(())
}

#[test]
fn get_elements_by_class_name_works() -> Result<()> {
    let html = r#"
        <ul>
          <li id='x' class='item target'>A</li>
          <li id='y' class='item'>B</li>
          <li id='z' class='target'>C</li>
          <li id='w' class='item target'>D</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const items = document.getElementsByClassName('item target');
            document.getElementById('result').textContent = items.length + ':' + items[0].id + ':' + items[1].id;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "2:x:w")?;
    Ok(())
}

#[test]
fn document_get_elements_by_class_name_is_live() -> Result<()> {
    let html = r#"
        <div id='main'>
          <span id='a' class='test red'>A</span>
          <span id='b' class='test'>B</span>
          <span id='c' class='red'>C</span>
        </div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const list = document.getElementsByClassName('test');
            const before = list.length + ':' + list[0].id + ':' + list[1].id;

            document.getElementById('b').className = 'plain';
            const afterRemove = list.length + ':' + list[0].id;

            const d = document.createElement('span');
            d.id = 'd';
            d.className = 'test';
            document.getElementById('main').appendChild(d);
            const afterAdd = list.length + ':' + list[1].id;

            document.getElementById('result').textContent =
              before + '|' + afterRemove + '|' + afterAdd;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "2:a:b|1:a|2:d")?;
    Ok(())
}

#[test]
fn element_class_name_reflects_class_attribute_and_absence() -> Result<()> {
    let html = r#"
        <div id='item'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const el = document.getElementById('item');
            const initial = el.className + ':' + (el.getAttribute('class') === null);

            el.className = 'active primary';
            const afterSet = el.className + ':' + el.getAttribute('class') + ':' + el.hasAttribute('class');

            el.removeAttribute('class');
            const afterRemove = el.className + ':' + (el.getAttribute('class') === null);

            document.getElementById('result').textContent =
              initial + '|' + afterSet + '|' + afterRemove;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", ":true|active primary:active primary:true|:true")?;
    Ok(())
}

#[test]
fn element_class_name_setter_coerces_to_string() -> Result<()> {
    let html = r#"
        <div id='item' class='active'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const el = document.getElementById('item');
            el.className = el.className === 'active' ? 'inactive' : 'active';
            const toggled = el.className + ':' + el.getAttribute('class');

            el.className = null;
            const coerced = el.className + ':' + el.getAttribute('class');

            document.getElementById('result').textContent = toggled + '|' + coerced;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "inactive:inactive|null:null")?;
    Ok(())
}

#[test]
fn svg_element_class_name_reflects_class_attribute_as_string() -> Result<()> {
    let html = r#"
        <svg id='icon'></svg>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const icon = document.getElementById('icon');
            icon.setAttribute('class', 'from-attr');
            const fromAttr = icon.className;

            icon.className = 'from-prop';
            const fromProp = icon.getAttribute('class');

            document.getElementById('result').textContent = fromAttr + ':' + fromProp;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "from-attr:from-prop")?;
    Ok(())
}

#[test]
fn element_get_elements_by_class_name_scopes_to_descendants_and_multiple_classes() -> Result<()> {
    let html = r#"
        <div id='main' class='test'>
          <p id='inside' class='test'>hello</p>
          <section><p id='deep' class='test red'>world</p></section>
        </div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const parent = document.getElementById('main');
            const list = parent.getElementsByClassName('test');
            const multi = parent.getElementsByClassName('test red');

            document.getElementById('result').textContent = [
              list.length,
              list[0].id,
              list[1].id,
              multi.length,
              multi[0].id
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "2:inside:deep:1:deep")?;
    Ok(())
}

#[test]
fn document_get_elements_by_class_name_empty_string_returns_empty_collection() -> Result<()> {
    let html = r#"
        <div class='test'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const list = document.getElementsByClassName('');
            document.getElementById('result').textContent =
              list.length + ':' + (list[0] === undefined);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "0:true")?;
    Ok(())
}

#[test]
fn parsed_document_get_elements_by_class_name_is_live() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const parsed = Document.parseHTML(
              '<div id="root"><p id="a" class="x y">A</p><p id="b" class="x">B</p></div>'
            );
            const list = parsed.getElementsByClassName('x');
            const root = parsed.getElementById('root');
            const before = list.length + ':' + list[0].id + ':' + list[1].id;

            parsed.getElementById('b').setAttribute('class', 'none');
            const afterRemove = list.length + ':' + list[0].id;

            const c = parsed.createElement('p');
            c.id = 'c';
            c.className = 'x';
            root.appendChild(c);
            const afterAdd = list.length + ':' + list[1].id;

            document.getElementById('result').textContent =
              before + '|' + afterRemove + '|' + afterAdd;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "2:a:b|1:a|2:c")?;
    Ok(())
}

#[test]
fn get_elements_by_tag_name_works() -> Result<()> {
    let html = r#"
        <ul>
          <li id='a'>A</li>
          <li id='b'>B</li>
        </ul>
        <section id='s'>
          <li id='c'>C</li>
        </section>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const items = document.getElementsByTagName('li');
            document.getElementById('result').textContent = items.length + ':' + items[2].id;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "3:c")?;
    Ok(())
}

#[test]
fn document_get_elements_by_tag_name_is_live_and_lowercases_argument() -> Result<()> {
    let html = r#"
        <p id='a'>A</p>
        <p id='b'>B</p>
        <div id='host'><p id='c'>C</p></div>
        <button id='run'>run</button>
        <div id='result'></div>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const list = document.getElementsByTagName('P');
            const before = list.length + ':' + list[0].id + ':' + list[1].id + ':' + list[2].id;

            document.getElementById('b').remove();
            const afterRemove = list.length + ':' + list[0].id + ':' + list[1].id;

            const added = document.createElement('P');
            added.id = 'd';
            document.getElementById('host').appendChild(added);
            const afterAdd = list.length + ':' + list[2].id;

            document.getElementById('result').textContent =
              before + '|' + afterRemove + '|' + afterAdd;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "3:a:b:c|2:a:c|3:d")?;
    Ok(())
}

#[test]
fn document_get_elements_by_tag_name_wildcard_returns_all_elements_in_tree_order() -> Result<()> {
    let html = r#"
        <div id='a'><span id='b'></span></div>
        <section id='c'><p id='d'></p></section>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const all = document.getElementsByTagName('*');
            const ids = [];
            for (const el of all) {
              if (el.id) ids.push(el.id);
            }
            document.getElementById('result').textContent = ids.join(',');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "a,b,c,d,run,result")?;
    Ok(())
}

#[test]
fn element_get_elements_by_tag_name_scopes_descendants_and_is_live() -> Result<()> {
    let html = r#"
        <div id='main'>
          <p id='p1'></p>
          <section id='sec'><p id='p2'></p></section>
        </div>
        <p id='outside'></p>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const main = document.getElementById('main');
            const all = main.getElementsByTagName('*');
            const ps = main.getElementsByTagName('p');
            const before = all.length + ':' + ps.length;

            const added = document.createElement('span');
            added.id = 'new-node';
            document.getElementById('sec').appendChild(added);
            const after = all.length + ':' + all[3].id + ':' + ps.length;

            document.getElementById('result').textContent = before + '|' + after;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "3:2|4:new-node:2")?;
    Ok(())
}

#[test]
fn parsed_document_get_elements_by_tag_name_is_live() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const parsed = Document.parseHTML('<div id="root"><p id="a"></p><p id="b"></p></div>');
            const root = parsed.getElementById('root');
            const list = parsed.getElementsByTagName('P');
            const before = list.length + ':' + list[0].id + ':' + list[1].id;

            parsed.getElementById('b').remove();
            const afterRemove = list.length + ':' + list[0].id;

            const c = parsed.createElement('p');
            c.id = 'c';
            root.appendChild(c);
            const afterAdd = list.length + ':' + list[1].id;

            document.getElementById('result').textContent =
              before + '|' + afterRemove + '|' + afterAdd;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "2:a:b|1:a|2:c")?;
    Ok(())
}

#[test]
fn get_elements_by_name_works() -> Result<()> {
    let html = r#"
        <input id='a' name='target' value='one'>
        <input id='b' name='other' value='other'>
        <input id='c' name='target' value='two'>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const fields = document.getElementsByName('target');
            document.getElementById('result').textContent = fields.length + ':' + fields[1].value;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "2:two")?;
    Ok(())
}

#[test]
fn document_get_elements_by_name_includes_matching_named_elements() -> Result<()> {
    let html = r#"
        <input id='u1' name='up'>
        <object id='u2' name='up'></object>
        <div id='u3' name='up'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const matches = document.getElementsByName('up');
            document.getElementById('result').textContent = [
              matches.length,
              matches[0].tagName,
              matches[1].tagName,
              matches[2].tagName
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "3:INPUT:OBJECT:DIV")?;
    Ok(())
}

#[test]
fn document_get_elements_by_name_is_live() -> Result<()> {
    let html = r#"
        <input id='u1' name='up'>
        <div id='u2' name='up'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const matches = document.getElementsByName('up');
            const before = matches.length + ':' + matches[0].id + ':' + matches[1].id;

            document.getElementById('u2').setAttribute('name', 'down');
            const afterRename = matches.length + ':' + matches[0].id;

            const added = document.createElement('input');
            added.id = 'u3';
            added.setAttribute('name', 'up');
            document.body.appendChild(added);
            const afterAdd = matches.length + ':' + matches[1].id;

            document.getElementById('u1').remove();
            const afterRemove = matches.length + ':' + matches[0].id;

            document.getElementById('result').textContent =
              before + '|' + afterRename + '|' + afterAdd + '|' + afterRemove;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "2:u1:u2|1:u1|2:u3|1:u3")?;
    Ok(())
}

#[test]
fn parsed_document_get_elements_by_name_is_live() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const parsed = Document.parseHTML(
              '<div id="root"><input id="a" name="up"><div id="b" name="up"></div></div>'
            );
            const root = parsed.getElementById('root');
            const matches = parsed.getElementsByName('up');
            const before = matches.length + ':' + matches[0].id + ':' + matches[1].id;

            parsed.getElementById('b').setAttribute('name', 'down');
            const afterRename = matches.length + ':' + matches[0].id;

            const added = parsed.createElement('span');
            added.id = 'c';
            added.setAttribute('name', 'up');
            root.appendChild(added);
            const afterAdd = matches.length + ':' + matches[1].id;

            document.getElementById('result').textContent =
              before + '|' + afterRename + '|' + afterAdd;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "2:a:b|1:a|2:c")?;
    Ok(())
}

#[test]
fn document_get_element_by_id_is_case_sensitive_and_returns_null_for_missing() -> Result<()> {
    let html = r#"
        <div id='main'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('result').textContent = [
              document.getElementById('main') !== null,
              document.getElementById('Main') === null,
              document.getElementById('missing') === null
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
fn document_get_element_by_id_does_not_find_detached_nodes_until_connected() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const detached = document.createElement('div');
            detached.id = 'ghost';
            const before = document.getElementById('ghost') === null;

            document.body.appendChild(detached);
            const after = document.getElementById('ghost');

            document.getElementById('result').textContent = [
              before,
              after !== null,
              after === detached
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
fn document_get_element_by_id_returns_first_element_in_document_order_for_duplicates() -> Result<()>
{
    let html = r#"
        <div id='first'></div>
        <div id='second'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const first = document.getElementById('first');
            const second = document.getElementById('second');
            second.id = 'dup';
            first.id = 'dup';

            const found = document.getElementById('dup');
            const which = found === first ? 'first' : (found === second ? 'second' : 'none');

            document.getElementById('result').textContent =
              which + ':' + document.querySelectorAll('#dup').length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "first:2")?;
    Ok(())
}

#[test]
fn get_element_by_id_is_not_available_on_element_nodes() -> Result<()> {
    let html = r#"
        <div id='parent-id'>
          <p>hello word1</p>
          <p id='test1'>hello word2</p>
          <p>hello word3</p>
          <p>hello word4</p>
        </div>
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const parentDOM = document.getElementById('parent-id');
            parentDOM.getElementById('test1');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("not a function"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}

#[test]
fn class_list_add_remove_multiple_arguments_work() -> Result<()> {
    let html = r#"
        <div id='box' class='base'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            box.classList.add('alpha', 'beta', 'gamma');
            box.classList.remove('base', 'gamma');
            document.getElementById('result').textContent = box.className;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "alpha beta")?;
    Ok(())
}

#[test]
fn class_list_assignment_forwards_to_class_attribute_value() -> Result<()> {
    let html = r#"
        <div id='box'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            box.classList = 'foo bar';
            const first = box.className + ':' + box.getAttribute('class') + ':' + box.classList.length;

            box.className = 'solo';
            const second = box.classList.length + ':' + box.classList.contains('solo');

            box.classList = null;
            const third = box.className + ':' + box.getAttribute('class') + ':' + box.classList.length;

            document.getElementById('result').textContent = first + '|' + second + '|' + third;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "foo bar:foo bar:2|1:true|null:null:1")?;
    Ok(())
}

#[test]
fn class_list_replace_method_updates_tokens() -> Result<()> {
    let html = r#"
        <div id='box' class='foo baz'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            box.classList.replace('foo', 'bar');
            const first = box.className + ':' + box.classList.contains('bar') + ':' + box.classList.contains('foo');

            box.classList.replace('missing', 'next');
            const second = box.className;

            box.classList.replace('bar', 'baz');
            const third = box.className;

            document.getElementById('result').textContent = first + '|' + second + '|' + third;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "bar baz:true:false|bar baz|baz")?;
    Ok(())
}

#[test]
fn member_chain_dom_targets_support_class_list_listener_and_open_property() -> Result<()> {
    let html = r#"
        <div id='dialog' class='panel hidden'></div>
        <button id='open-tool'>open</button>
        <details id='settings' open></details>
        <p id='result'></p>
        <script>
          const el = {
            dialog: document.getElementById('dialog'),
            openToolBtn: document.getElementById('open-tool'),
            settingsDetails: document.getElementById('settings'),
          };

          el.dialog.classList.remove('hidden');
          el.openToolBtn.addEventListener('click', () => {
            document.getElementById('result').textContent =
              el.dialog.className + ':' + (el.settingsDetails.open ? 'open' : 'closed');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#open-tool")?;
    h.assert_text("#result", "panel:open")?;
    Ok(())
}

#[test]
fn document_core_properties_and_collections_work() -> Result<()> {
    let html = r#"
        <html id='doc'>
          <head id='head'>
            <title>Initial</title>
          </head>
          <body id='body'>
            <form id='f'><input id='name'></form>
            <img id='logo' src='logo.png'>
            <a id='link' href='/x'>x</a>
            <button id='btn'>run</button>
            <p id='result'></p>
            <script>
              document.getElementById('btn').addEventListener('click', () => {
                const kids = document.children;
                const first = document.firstElementChild;
                const last = document.lastElementChild;
                const activeBeforeNode = document.activeElement;
                const activeBefore = activeBeforeNode ? activeBeforeNode.id : 'none';
                document.getElementById('name').focus();
                const activeAfterNode = document.activeElement;
                const activeAfter = activeAfterNode ? activeAfterNode.id : 'none';

                document.getElementById('result').textContent =
                  document.title + ':' +
                  document.characterSet + ':' +
                  document.compatMode + ':' +
                  document.contentType + ':' +
                  document.readyState + ':' +
                  document.referrer + ':' +
                  document.URL + ':' +
                  document.documentURI + ':' +
                  document.location + ':' +
                  document.location.href + ':' +
                  document.visibilityState + ':' +
                  document.hidden + ':' +
                  document.body.id + ':' +
                  document.head.id + ':' +
                  document.documentElement.id + ':' +
                  document.childElementCount + ':' +
                  kids.length + ':' +
                  first.id + ':' +
                  last.id + ':' +
                  document.forms.length + ':' +
                  document.images.length + ':' +
                  document.links.length + ':' +
                  document.scripts.length + ':' +
                  activeBefore + ':' +
                  activeAfter + ':' +
                  (document.defaultView ? 'yes' : 'no');
              });
            </script>
          </body>
        </html>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "Initial:UTF-8:CSS1Compat:text/html:complete::about:blank:about:blank:about:blank:about:blank:visible:false:body:head:doc:1:1:doc:doc:1:1:1:1:body:name:yes",
    )?;
    Ok(())
}

#[test]
fn element_constructor_global_and_instanceof_work() -> Result<()> {
    let html = r#"
        <div id='box'></div>
        <input id='field' />
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const box = document.getElementById('box');
            const field = document.getElementById('field');

            document.getElementById('result').textContent = [
              typeof Element,
              window.Element === Element,
              box instanceof Element,
              box instanceof HTMLElement,
              box instanceof HTMLInputElement,
              field instanceof Element,
              field instanceof HTMLElement,
              field instanceof HTMLInputElement,
              document instanceof Element
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "function:true:true:true:false:true:true:true:false",
    )?;
    Ok(())
}

#[test]
fn document_title_assignment_and_body_chain_target_work() -> Result<()> {
    let html = r#"
        <html id='doc'>
          <head id='head'></head>
          <body id='body'>
            <button id='btn'>run</button>
            <p id='result'></p>
            <script>
              document.body.classList.add('ready');
              document.body.addEventListener('click', () => {});

              document.getElementById('btn').addEventListener('click', () => {
                document.title = 'Updated';
                const first = document.firstElementChild;
                const last = document.lastElementChild;
                document.getElementById('result').textContent =
                  document.title + ':' +
                  document.head.id + ':' +
                  document.documentElement.id + ':' +
                  document.body.className + ':' +
                  first.id + ':' +
                  last.id;
              });
            </script>
          </body>
        </html>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "Updated:head:doc:ready:doc:doc")?;
    Ok(())
}

#[test]
fn document_create_element_member_call_supports_dynamic_tag_name() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const tag = 'section';
            const node = document.createElement(tag);
            node.id = 'dynamic-node';
            node.textContent = 'ok';
            document.body.appendChild(node);
            const tail = document.body.lastElementChild;

            document.getElementById('result').textContent =
              node.tagName + ':' +
              (document.getElementById('dynamic-node') === node) + ':' +
              tail.id;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "SECTION:true:dynamic-node")?;
    Ok(())
}

#[test]
fn document_create_element_supports_options_is_and_legacy_string() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const modern = document.createElement('ul', { is: 'expanding-list' });
            const legacy = document.createElement('ul', 'legacy-list');
            const plain = document.createElement('ul', {});

            document.getElementById('result').textContent = [
              modern.getAttribute('is'),
              legacy.getAttribute('is'),
              plain.getAttribute('is') === null
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "expanding-list:legacy-list:true")?;
    Ok(())
}

#[test]
fn document_create_element_null_local_name_is_stringified() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const el = document.createElement(null);
            document.getElementById('result').textContent = el.tagName + ':' + el.localName;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "NULL:null")?;
    Ok(())
}

#[test]
fn parsed_document_create_element_supports_options_is() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const parsed = Document.parseHTML('<div id="root"></div>');
            const node = parsed.createElement('ul', { is: 'expanding-list' });
            parsed.getElementById('root').appendChild(node);

            document.getElementById('result').textContent =
              node.getAttribute('is') + ':' +
              parsed.querySelector('ul').getAttribute('is');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "expanding-list:expanding-list")?;
    Ok(())
}

#[test]
fn document_constructor_and_static_parse_html_methods_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const fresh = new Document();
            const called = Document();

            const safe = Document.parseHTML(
              '<div id="x" onclick="evil()"><script id="s">x</script><a id="a" href="javascript:alert(1)">go</a></div>'
            );
            const unsafe = Document.parseHTMLUnsafe(
              '<div id="x" onclick="evil()"><script id="s">x</script><a id="a" href="javascript:alert(1)">go</a></div>'
            );

            const made = fresh.createElement('span');
            made.appendChild(fresh.createTextNode('<b>x</b>'));

            document.getElementById('result').textContent = [
              typeof Document,
              fresh !== document,
              fresh.body === null,
              called.body === null,
              safe.querySelectorAll('#s').length,
              safe.querySelector('#x').hasAttribute('onclick'),
              safe.querySelector('#a').hasAttribute('href'),
              unsafe.querySelectorAll('#s').length,
              unsafe.querySelector('#x').hasAttribute('onclick'),
              unsafe.querySelector('#a').hasAttribute('href'),
              made.textContent
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "function:true:true:true:0:false:false:1:true:true:<b>x</b>",
    )?;
    Ok(())
}

#[test]
fn element_set_attribute_sets_updates_and_coerces_values() -> Result<()> {
    let html = r#"
        <div>
          <button id="hello_button" type="button">Some Text</button>
          <button id="run">run</button>
        </div>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const helloButton = document.getElementById('hello_button');
            helloButton.setAttribute('NAME', 'helloButton');
            helloButton.innerText = helloButton.getAttribute('name');

            helloButton.setAttribute('data-count', 42);
            helloButton.setAttribute('data-null', null);
            helloButton.setAttribute('disabled', 'disabled');
            const disabledAfterSet = helloButton.disabled;
            helloButton.removeAttribute('disabled');

            document.getElementById('result').textContent = [
              helloButton.getAttribute('name'),
              helloButton.innerText,
              helloButton.getAttribute('data-count'),
              helloButton.getAttribute('data-null'),
              disabledAfterSet,
              helloButton.disabled
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "helloButton:helloButton:42:null:true:false")?;
    Ok(())
}

#[test]
fn element_remove_attribute_removes_existing_attribute_and_returns_undefined() -> Result<()> {
    let html = r#"
        <div id='box' disabled data-keep='v'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const box = document.getElementById('box');
            const ret = box.removeAttribute('DISABLED');
            const retMissing = box.removeAttribute('missing');
            document.getElementById('result').textContent = [
              String(ret),
              String(retMissing),
              box.hasAttribute('disabled'),
              box.getAttribute('data-keep')
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "undefined:undefined:false:v")?;
    Ok(())
}

#[test]
fn element_remove_attribute_is_noop_when_attribute_is_absent() -> Result<()> {
    let html = r#"
        <div id='box' data-flag='on'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const box = document.getElementById('box');
            box.removeAttribute('not-there');
            document.getElementById('result').textContent = [
              box.getAttribute('data-flag'),
              box.hasAttribute('not-there')
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "on:false")?;
    Ok(())
}

#[test]
fn element_remove_attribute_rejects_non_single_argument_count() -> Result<()> {
    let html = r#"
        <div id='box'></div>
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('box').removeAttribute('id', 'extra');
          });
        </script>
        "#;

    match Harness::from_html(html) {
        Err(Error::ScriptParse(message)) => {
            assert!(
                message.contains("removeAttribute requires exactly one argument"),
                "unexpected parse error message: {message}"
            );
        }
        other => panic!("expected parse error, got: {other:?}"),
    }
    Ok(())
}

#[test]
fn element_remove_removes_connected_node_and_is_noop_when_called_again() -> Result<()> {
    let html = r#"
        <div id='div-01'>Here is div-01</div>
        <div id='div-02'>Here is div-02</div>
        <div id='div-03'>Here is div-03</div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const element = document.getElementById('div-02');
            const first = element.remove();
            const second = element.remove();
            document.getElementById('result').textContent = [
              String(first),
              String(second),
              document.getElementById('div-02') === null,
              document.getElementsByTagName('div').length
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "undefined:undefined:true:2")?;
    Ok(())
}

#[test]
fn element_remove_rejects_arguments_in_expression_context() -> Result<()> {
    let html = r#"
        <div id='box'></div>
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const out = document.getElementById('box').remove(1);
            document.getElementById('box').textContent = String(out);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("remove takes no arguments"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}

#[test]
fn element_get_attribute_lowercases_argument_and_returns_null_when_missing() -> Result<()> {
    let html = r#"
        <div id='box' data-count='42'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const box = document.getElementById('box');
            document.getElementById('result').textContent = [
              box.getAttribute('ID'),
              box.getAttribute('DATA-COUNT'),
              box.getAttribute('missing') === null
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "box:42:true")?;
    Ok(())
}

#[test]
fn element_get_attribute_hides_nonce_but_nonce_property_still_returns_value() -> Result<()> {
    let html = r#"
        <script id='s' nonce='abc123'></script>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const s = document.getElementById('s');
            document.getElementById('result').textContent = [
              s.getAttribute('nonce') === '',
              s.nonce,
              s.getAttribute('NONCE') === '',
              s.getAttribute('missing') === null
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:abc123:true:true")?;
    Ok(())
}

#[test]
fn element_has_attribute_returns_boolean_and_is_case_insensitive() -> Result<()> {
    let html = r#"
        <div id='box' data-flag='on'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const box = document.getElementById('box');
            document.getElementById('result').textContent = [
              box.hasAttribute('data-flag'),
              box.hasAttribute('DATA-FLAG'),
              box.hasAttribute('missing')
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:true:false")?;
    Ok(())
}

#[test]
fn element_has_attribute_coerces_non_string_arguments() -> Result<()> {
    let html = r#"
        <div id='box'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const box = document.getElementById('box');
            box.setAttribute('null', 'v1');
            box.setAttribute('undefined', 'v2');
            box.setAttribute('true', 'v3');

            document.getElementById('result').textContent = [
              box.hasAttribute(null),
              box.hasAttribute(undefined),
              box.hasAttribute(true),
              box.hasAttribute(false)
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
fn element_has_attribute_rejects_non_single_argument_count() -> Result<()> {
    let html = r#"
        <div id='box'></div>
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('box').hasAttribute('id', 'extra');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("hasAttribute requires exactly one argument"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}

#[test]
fn element_has_attributes_returns_boolean_for_attribute_presence() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const empty = document.createElement('div');
            const withOne = document.createElement('div');
            withOne.setAttribute('data-flag', 'on');
            const beforeRemove = withOne.hasAttributes();
            withOne.removeAttribute('data-flag');
            const afterRemove = withOne.hasAttributes();

            empty.id = 'tmp';
            const idOnly = empty.hasAttributes();

            document.getElementById('result').textContent = [
              document.createElement('span').hasAttributes(),
              beforeRemove,
              afterRemove,
              idOnly
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "false:true:false:true")?;
    Ok(())
}

#[test]
fn element_has_attributes_rejects_arguments() -> Result<()> {
    let html = r#"
        <div id='box'></div>
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('box').hasAttributes('extra');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("hasAttributes takes no arguments"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}

#[test]
fn element_toggle_attribute_toggles_and_lowercases_attribute_name() -> Result<()> {
    let html = r#"
        <div id='box'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const box = document.getElementById('box');
            const first = box.toggleAttribute('DATA-FLAG');
            const firstHas = box.hasAttribute('data-flag');
            const firstEmpty = box.getAttribute('data-flag') === '';
            const second = box.toggleAttribute('data-flag');
            document.getElementById('result').textContent = [
              first,
              firstHas,
              firstEmpty,
              second,
              box.hasAttribute('data-flag')
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:true:true:false:false")?;
    Ok(())
}

#[test]
fn element_toggle_attribute_force_argument_controls_presence() -> Result<()> {
    let html = r#"
        <input id='box'>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const box = document.getElementById('box');
            const add = box.toggleAttribute('disabled', true);
            const addAgain = box.toggleAttribute('disabled', true);
            const remove = box.toggleAttribute('disabled', false);
            const removeAgain = box.toggleAttribute('disabled', false);
            document.getElementById('result').textContent = [
              add,
              addAgain,
              remove,
              removeAgain,
              box.hasAttribute('disabled')
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:true:false:false:false")?;
    Ok(())
}

#[test]
fn element_toggle_attribute_rejects_invalid_name() -> Result<()> {
    let html = r#"
        <div id='box'></div>
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const bad = '1bad';
            document.getElementById('box').toggleAttribute(bad);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("InvalidCharacterError"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}

#[test]
fn element_set_attribute_rejects_invalid_name_literal_argument() -> Result<()> {
    let html = r#"
        <div id='box'></div>
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('box').setAttribute('1bad', 'x');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("InvalidCharacterError"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}

#[test]
fn element_set_attribute_rejects_invalid_name_dynamic_argument() -> Result<()> {
    let html = r#"
        <div id='box'></div>
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const bad = '1bad';
            document.getElementById('box').setAttribute(bad, 'x');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("InvalidCharacterError"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}

#[test]
fn document_create_attribute_and_set_attribute_node_work() -> Result<()> {
    let html = r#"
        <div id='div1'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const node = document.getElementById('div1');

            const first = document.createAttribute('MY_ATTR');
            const firstName = first.name;
            const firstOwnerBefore = first.ownerElement === null;
            first.value = 'newVal';
            const replacedFirst = node.setAttributeNode(first);

            const second = document.createAttribute('my_attr');
            second.value = 'newer';
            const replacedSecond = node.setAttributeNode(second);

            document.getElementById('result').textContent = [
              firstName,
              firstOwnerBefore,
              node.getAttribute('my_attr'),
              replacedFirst === null,
              replacedSecond !== null ? replacedSecond.value : 'none',
              second.ownerElement === node
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "my_attr:true:newer:true:newVal:true")?;
    Ok(())
}

#[test]
fn parsed_document_create_attribute_method_works() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const parsed = Document.parseHTML('<div id="x"></div>');
            const attr = parsed.createAttribute('DATA_X');
            attr.value = 'ok';
            const target = parsed.querySelector('#x');
            target.setAttributeNode(attr);

            document.getElementById('result').textContent =
              parsed.querySelector('#x').getAttribute('data_x') + ':' +
              attr.name;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "ok:data_x")?;
    Ok(())
}

#[test]
fn document_create_attribute_rejects_invalid_name() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.createAttribute('1bad');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("InvalidCharacterError"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}

#[test]
fn document_create_document_fragment_builds_offscreen_tree_and_appends_children() -> Result<()> {
    let html = r#"
        <ul id='ul'></ul>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const element = document.getElementById('ul');
            const fragment = document.createDocumentFragment();
            const browsers = ['Firefox', 'Chrome', 'Opera', 'Safari'];

            browsers.forEach((browser) => {
              const li = document.createElement('li');
              li.textContent = browser;
              fragment.appendChild(li);
            });

            const beforeAppend = fragment.childNodes.length;
            element.appendChild(fragment);

            document.getElementById('result').textContent = [
              fragment.nodeType,
              fragment.nodeName,
              beforeAppend,
              fragment.childNodes.length,
              element.children.length,
              element.firstElementChild.textContent,
              element.lastElementChild.textContent
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "11:#document-fragment:4:0:4:Firefox:Safari")?;
    Ok(())
}

#[test]
fn parsed_document_create_document_fragment_method_works() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const parsed = Document.parseHTML('<div id="root"></div>');
            const fragment = parsed.createDocumentFragment();
            const child = parsed.createElement('span');
            child.textContent = 'A';
            fragment.appendChild(child);

            const root = parsed.getElementById('root');
            const returned = root.appendChild(fragment);

            document.getElementById('result').textContent = [
              fragment.nodeType,
              returned === fragment,
              fragment.childNodes.length,
              root.childNodes.length,
              root.firstChild.textContent
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "11:true:0:1:A")?;
    Ok(())
}

#[test]
fn document_create_range_defaults_to_document_and_setters_work() -> Result<()> {
    let html = r#"
        <p id='host'></p>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const range = document.createRange();
            const initial = [
              range.startContainer.nodeType,
              range.startOffset,
              range.endContainer.nodeType,
              range.endOffset
            ].join(':');

            const text = document.createTextNode('ABCDE');
            document.getElementById('host').appendChild(text);
            range.setStart(text, 1);
            range.setEnd(text, 4);

            const after = [
              range.startContainer === text,
              range.startOffset,
              range.endContainer === text,
              range.endOffset
            ].join(':');

            document.getElementById('result').textContent = initial + '|' + after;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "9:0:9:0|true:1:true:4")?;
    Ok(())
}

#[test]
fn parsed_document_create_range_method_works() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const parsed = Document.parseHTML('<p id="x">abcde</p>');
            const range = parsed.createRange();
            const parsedDocNode = parsed.documentElement.parentNode;
            const initial = range.startContainer === parsedDocNode && range.endContainer === parsedDocNode;

            const text = parsed.getElementById('x').firstChild;
            range.setStart(text, 2);
            range.setEnd(text, 5);

            document.getElementById('result').textContent = [
              initial,
              range.startContainer === text,
              range.startOffset,
              range.endContainer === text,
              range.endOffset
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:true:2:true:5")?;
    Ok(())
}

#[test]
fn document_append_allows_new_document_root_element() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const doc = new Document();
            const htmlRoot = document.createElement('html');
            const returned = doc.append(htmlRoot);
            document.getElementById('result').textContent = [
              returned === undefined,
              doc.querySelectorAll('html').length,
              doc.documentElement === htmlRoot
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:1:true")?;
    Ok(())
}

#[test]
fn document_append_throws_hierarchy_request_error_for_existing_root_element() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const htmlRoot = document.createElement('html');
            document.append(htmlRoot);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("HierarchyRequestError"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}

#[test]
fn document_append_throws_hierarchy_request_error_for_string_argument() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.append('text');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("HierarchyRequestError"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}

#[test]
fn location_properties_and_setters_work_from_location_document_and_window() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            location.href = 'https://developer.mozilla.org:8080/en-US/search?q=URL#search-results-close-container';
            document.location.protocol = 'http:';
            window.location.hostname = 'example.com';
            location.port = '9090';
            location.pathname = 'docs';
            location.search = 'k=v';
            location.hash = 'anchor';

            document.getElementById('result').textContent =
              location.href + '|' +
              location.protocol + '|' +
              location.host + '|' +
              location.hostname + '|' +
              location.port + '|' +
              location.pathname + '|' +
              location.search + '|' +
              location.hash + '|' +
              location.origin + '|' +
              document.location.toString() + '|' +
              window.location.toString() + '|' +
              location.ancestorOrigins.length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "http://example.com:9090/docs?k=v#anchor|http:|example.com:9090|example.com|9090|/docs|?k=v|#anchor|http://example.com:9090|http://example.com:9090/docs?k=v#anchor|http://example.com:9090/docs?k=v#anchor|0",
    )?;
    Ok(())
}

#[test]
fn location_file_host_setters_follow_url_semantics_and_skip_noop_navigation_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            location.href = 'file://server/share/file.txt';
            const a = [location.href, location.host, location.hostname, location.port].join(',');

            location.port = '8080';
            const b = [location.href, location.host, location.hostname, location.port].join(',');

            location.hostname = 'example.com';
            const c = [location.href, location.host, location.hostname, location.port].join(',');

            location.host = 'localhost';
            const d = [location.href, location.host, location.hostname, location.port].join(',');

            location.host = 'localhost:8080';
            const e = [location.href, location.host, location.hostname, location.port].join(',');

            document.getElementById('result').textContent = [a, b, c, d, e].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "file://server/share/file.txt,server,server,|file://server/share/file.txt,server,server,|file://example.com/share/file.txt,example.com,example.com,|file:///share/file.txt,,,|file:///share/file.txt,,,",
    )?;
    assert_eq!(
        h.take_location_navigations(),
        vec![
            LocationNavigation {
                kind: LocationNavigationKind::HrefSet,
                from: "about:blank".to_string(),
                to: "file://server/share/file.txt".to_string(),
            },
            LocationNavigation {
                kind: LocationNavigationKind::HrefSet,
                from: "file://server/share/file.txt".to_string(),
                to: "file://example.com/share/file.txt".to_string(),
            },
            LocationNavigation {
                kind: LocationNavigationKind::HrefSet,
                from: "file://example.com/share/file.txt".to_string(),
                to: "file:///share/file.txt".to_string(),
            },
        ]
    );
    Ok(())
}

#[test]
fn location_file_invalid_authority_inputs_throw_and_do_not_navigate_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const outcomes = [];
            const capture = (label, action) => {
              try {
                action();
                outcomes.push(label + ':false');
              } catch (err) {
                outcomes.push(label + ':' + String(err).includes('Invalid URL'));
              }
            };

            capture('href', () => {
              location.href = 'file://server:8080/share/file.txt';
            });
            capture('assign', () => {
              location.assign('file://u:p@server/share/file.txt');
            });
            capture('replace', () => {
              location.replace('file://localhost:8080/Users/me/test.txt');
            });
            capture('document', () => {
              document.location.href = 'file://u@server/share/file.txt';
            });
            capture('window', () => {
              window.location.href = 'file://:p@server/share/file.txt';
            });

            document.getElementById('result').textContent =
              outcomes.join('|') + '|' + location.href;
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/start", html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "href:true|assign:true|replace:true|document:true|window:true|https://app.local/start",
    )?;
    assert!(h.take_location_navigations().is_empty());
    Ok(())
}

#[test]
fn file_url_document_serialization_and_location_alias_noop_parity_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const before = [
              document.URL,
              document.documentURI,
              document.location.href,
              window.location.href,
              document.location.origin,
              window.location.origin,
              navigation.currentEntry.url
            ].join(',');

            location.href = 'FiLe://SeRVer/Share/File.txt';
            const afterNavigate = [
              document.URL,
              document.documentURI,
              document.location.href,
              window.location.href,
              document.location.origin,
              window.location.origin,
              navigation.currentEntry.url
            ].join(',');

            document.location.hostname = 'SERVER';
            window.location.host = 'SERVER:8080';
            const afterNoop = [
              document.location.href,
              window.location.href,
              document.location.host,
              window.location.hostname,
              navigation.currentEntry.url
            ].join(',');

            document.getElementById('result').textContent =
              [before, afterNavigate, afterNoop].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("FiLe://LOCALHOST/Users/Me/Start/Index.html", html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "file:///Users/Me/Start/Index.html,file:///Users/Me/Start/Index.html,file:///Users/Me/Start/Index.html,file:///Users/Me/Start/Index.html,null,null,file:///Users/Me/Start/Index.html|file://server/Share/File.txt,file://server/Share/File.txt,file://server/Share/File.txt,file://server/Share/File.txt,null,null,file://server/Share/File.txt|file://server/Share/File.txt,file://server/Share/File.txt,server,server,file://server/Share/File.txt",
    )?;
    assert_eq!(
        h.take_location_navigations(),
        vec![LocationNavigation {
            kind: LocationNavigationKind::HrefSet,
            from: "file:///Users/Me/Start/Index.html".to_string(),
            to: "file://server/Share/File.txt".to_string(),
        }]
    );
    Ok(())
}

#[test]
fn location_and_history_file_idna_host_residuals_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            location.href = 'file://\u00E9xample.com/share/file.txt';
            const afterHref = [
              location.href,
              location.host,
              navigation.currentEntry.url,
              history.length
            ].join(',');

            location.host = 'example\u3002com.';
            const afterHost = [
              location.href,
              location.host,
              location.hostname,
              navigation.currentEntry.url,
              history.length
            ].join(',');

            history.replaceState({ step: 1 }, '', 'file://\u05D0.com/docs');
            const afterReplace = [
              location.href,
              document.URL,
              document.documentURI,
              location.host,
              location.pathname,
              navigation.currentEntry.url,
              history.length
            ].join(',');

            location.hostname = 'a\u200Db.com';
            const afterInvalidSetter = [
              location.href,
              location.host,
              location.hostname,
              navigation.currentEntry.url,
              history.length
            ].join(',');

            location.host = 'localhost';
            const afterLocalhost = [
              location.href,
              document.URL,
              document.documentURI,
              location.host,
              location.hostname,
              navigation.currentEntry.url,
              history.length
            ].join(',');

            document.getElementById('result').textContent = [
              afterHref,
              afterHost,
              afterReplace,
              afterInvalidSetter,
              afterLocalhost
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("file:///Users/Me/Start/Index.html", html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "file://xn--xample-9ua.com/share/file.txt,xn--xample-9ua.com,file://xn--xample-9ua.com/share/file.txt,2|file://example.com./share/file.txt,example.com.,example.com.,file://example.com./share/file.txt,3|file://xn--4db.com/docs,file://xn--4db.com/docs,file://xn--4db.com/docs,xn--4db.com,/docs,file://xn--4db.com/docs,3|file://xn--4db.com/docs,xn--4db.com,xn--4db.com,file://xn--4db.com/docs,3|file:///docs,file:///docs,file:///docs,,,file:///docs,4",
    )?;
    assert_eq!(
        h.take_location_navigations(),
        vec![
            LocationNavigation {
                kind: LocationNavigationKind::HrefSet,
                from: "file:///Users/Me/Start/Index.html".to_string(),
                to: "file://xn--xample-9ua.com/share/file.txt".to_string(),
            },
            LocationNavigation {
                kind: LocationNavigationKind::HrefSet,
                from: "file://xn--xample-9ua.com/share/file.txt".to_string(),
                to: "file://example.com./share/file.txt".to_string(),
            },
            LocationNavigation {
                kind: LocationNavigationKind::HrefSet,
                from: "file://xn--4db.com/docs".to_string(),
                to: "file:///docs".to_string(),
            },
        ]
    );
    Ok(())
}

#[test]
fn location_and_history_invalid_generic_authority_inputs_throw_and_do_not_navigate_work()
-> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const outcomes = [];
            const capture = (label, action) => {
              try {
                action();
                outcomes.push(label + ':false');
              } catch (err) {
                outcomes.push(label + ':' + String(err).includes('Invalid URL'));
              }
            };

            capture('href', () => {
              location.href = 'https://example.com:abc/path';
            });
            capture('assign', () => {
              location.assign('https://example.com:65536/path');
            });
            capture('push', () => {
              history.pushState({ step: 1 }, '', 'http://[::1/path');
            });
            capture('replace', () => {
              history.replaceState({ step: 2 }, '', 'https://example.com:99999/path');
            });

            document.getElementById('result').textContent =
              outcomes.join('|') + '|' + location.href + '|' + history.length;
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/start", html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "href:true|assign:true|push:true|replace:true|https://app.local/start|1",
    )?;
    assert!(h.take_location_navigations().is_empty());
    Ok(())
}

#[test]
fn location_and_history_special_host_inputs_canonicalize_and_empty_host_throw_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            let invalidEmpty = false;
            let invalidQuery = false;
            let invalidAuthority = false;

            location.href = 'https:Example.COM:080/path';
            const afterLocation = location.href;

            history.pushState({ step: 1 }, '', 'http:\\Example.COM\\next\\page?x=1#frag');
            const afterPush = [
              location.href,
              navigation.currentEntry.url,
              history.length,
              history.state.step
            ].join(',');

            history.replaceState({ step: 2 }, '', 'http://example.com:');
            const afterReplace = [
              location.href,
              navigation.currentEntry.url,
              history.length,
              history.state.step
            ].join(',');

            try {
              location.assign('http://');
            } catch (err) {
              invalidEmpty = String(err).includes('Invalid URL');
            }

            try {
              history.pushState({ step: 3 }, '', 'http:?x');
            } catch (err) {
              invalidQuery = String(err).includes('Invalid URL');
            }

            try {
              history.replaceState({ step: 4 }, '', 'http://?x');
            } catch (err) {
              invalidAuthority = String(err).includes('Invalid URL');
            }

            document.getElementById('result').textContent = [
              afterLocation,
              afterPush,
              afterReplace,
              invalidEmpty,
              invalidQuery,
              invalidAuthority,
              location.href,
              navigation.currentEntry.url,
              history.length,
              history.state.step
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/start", html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "https://example.com:80/path|http://example.com/next/page?x=1#frag,http://example.com/next/page?x=1#frag,3,1|http://example.com/,http://example.com/,3,2|true|true|true|http://example.com/|http://example.com/|3|2",
    )?;
    assert_eq!(
        h.take_location_navigations(),
        vec![LocationNavigation {
            kind: LocationNavigationKind::HrefSet,
            from: "https://app.local/start".to_string(),
            to: "https://example.com:80/path".to_string(),
        }]
    );
    Ok(())
}

#[test]
fn location_and_history_credentials_and_delimiter_inputs_canonicalize_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            location.href = 'https://a@b:p@q:r@example.com\\docs\\a b?a\'b#x`y';
            const afterLocation = [
              location.href,
              location.host,
              location.pathname,
              location.search,
              location.hash,
              navigation.currentEntry.url
            ].join(',');

            history.pushState({ step: 1 }, '', 'foo://example.com/\\docs\\a b?a\'b#x`y');
            const afterPush = [
              location.href,
              location.pathname,
              location.search,
              location.hash,
              navigation.currentEntry.url,
              history.length
            ].join(',');

            history.replaceState({ step: 2 }, '', 'file:///Users/me/base');
            location.pathname = '\\docs\\a b';
            location.search = "a'b";
            location.hash = 'x`y';
            const afterReplace = [
              location.href,
              location.pathname,
              location.search,
              location.hash,
              navigation.currentEntry.url,
              history.length
            ].join(',');

            document.getElementById('result').textContent = [
              afterLocation,
              afterPush,
              afterReplace
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/start", html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "https://a%40b:p%40q%3Ar@example.com/docs/a%20b?a%27b#x%60y,example.com,/docs/a%20b,?a%27b,#x%60y,https://a%40b:p%40q%3Ar@example.com/docs/a%20b?a%27b#x%60y|foo://example.com/\\docs\\a%20b?a'b#x%60y,/\\docs\\a%20b,?a'b,#x%60y,foo://example.com/\\docs\\a%20b?a'b#x%60y,3|file:///docs/a%20b?a%27b#x%60y,/docs/a%20b,?a%27b,#x%60y,file:///docs/a%20b?a%27b#x%60y,6",
    )?;
    assert_eq!(
        h.take_location_navigations(),
        vec![
            LocationNavigation {
                kind: LocationNavigationKind::HrefSet,
                from: "https://app.local/start".to_string(),
                to: "https://a%40b:p%40q%3Ar@example.com/docs/a%20b?a%27b#x%60y".to_string(),
            },
            LocationNavigation {
                kind: LocationNavigationKind::HrefSet,
                from: "file:///Users/me/base".to_string(),
                to: "file:///docs/a%20b".to_string(),
            },
            LocationNavigation {
                kind: LocationNavigationKind::HrefSet,
                from: "file:///docs/a%20b".to_string(),
                to: "file:///docs/a%20b?a%27b".to_string(),
            },
            LocationNavigation {
                kind: LocationNavigationKind::HrefSet,
                from: "file:///docs/a%20b?a%27b".to_string(),
                to: "file:///docs/a%20b?a%27b#x%60y".to_string(),
            },
        ]
    );
    Ok(())
}

#[test]
fn location_and_history_authority_and_percent_residuals_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const invalid = (() => {
              try {
                location.href = 'https://exa%mple.org/';
                return 'false';
              } catch (err) {
                return [
                  String(err).includes('Invalid URL'),
                  location.href,
                  navigation.currentEntry.url,
                  history.length
                ].join(',');
              }
            })();

            location.href = 'https://a@@ExA%41mple.ORG/%2f%zz?x=%2f%zz#y=%2f%zz';
            const afterHref = [
              location.href,
              location.host,
              location.pathname,
              location.search,
              location.hash,
              navigation.currentEntry.url,
              history.length
            ].join(',');

            location.host = 'exa%mple.org:77';
            const afterBadHost = [
              location.href,
              location.host,
              navigation.currentEntry.url,
              history.length
            ].join(',');

            location.host = '%41example.com:0099';
            const afterHost = [
              location.href,
              location.host,
              location.pathname,
              location.search,
              location.hash,
              navigation.currentEntry.url,
              history.length
            ].join(',');

            history.pushState({ step: 1 }, '', 'foo://example.com/%2f%zz?x=%2f%zz#y=%2f%zz');
            const afterPush = [
              location.href,
              location.pathname,
              location.search,
              location.hash,
              navigation.currentEntry.url,
              history.length
            ].join(',');

            history.replaceState({ step: 2 }, '', 'https://user:@example.com/%2f%zz?x=%2f%zz#y=%2f%zz');
            const afterReplace = [
              location.href,
              location.host,
              location.pathname,
              location.search,
              location.hash,
              navigation.currentEntry.url,
              history.length
            ].join(',');

            document.getElementById('result').textContent = [
              invalid,
              afterHref,
              afterBadHost,
              afterHost,
              afterPush,
              afterReplace
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/start", html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "true,https://app.local/start,https://app.local/start,1|https://a%40@exaample.org/%2f%zz?x=%2f%zz#y=%2f%zz,exaample.org,/%2f%zz,?x=%2f%zz,#y=%2f%zz,https://a%40@exaample.org/%2f%zz?x=%2f%zz#y=%2f%zz,2|https://a%40@exaample.org/%2f%zz?x=%2f%zz#y=%2f%zz,exaample.org,https://a%40@exaample.org/%2f%zz?x=%2f%zz#y=%2f%zz,2|https://a%40@aexample.com:99/%2f%zz?x=%2f%zz#y=%2f%zz,aexample.com:99,/%2f%zz,?x=%2f%zz,#y=%2f%zz,https://a%40@aexample.com:99/%2f%zz?x=%2f%zz#y=%2f%zz,3|foo://example.com/%2f%zz?x=%2f%zz#y=%2f%zz,/%2f%zz,?x=%2f%zz,#y=%2f%zz,foo://example.com/%2f%zz?x=%2f%zz#y=%2f%zz,4|https://user@example.com/%2f%zz?x=%2f%zz#y=%2f%zz,example.com,/%2f%zz,?x=%2f%zz,#y=%2f%zz,https://user@example.com/%2f%zz?x=%2f%zz#y=%2f%zz,4",
    )?;
    assert_eq!(
        h.take_location_navigations(),
        vec![
            LocationNavigation {
                kind: LocationNavigationKind::HrefSet,
                from: "https://app.local/start".to_string(),
                to: "https://a%40@exaample.org/%2f%zz?x=%2f%zz#y=%2f%zz".to_string(),
            },
            LocationNavigation {
                kind: LocationNavigationKind::HrefSet,
                from: "https://a%40@exaample.org/%2f%zz?x=%2f%zz#y=%2f%zz".to_string(),
                to: "https://a%40@aexample.com:99/%2f%zz?x=%2f%zz#y=%2f%zz".to_string(),
            },
        ]
    );
    Ok(())
}

#[test]
fn location_and_history_malformed_query_and_host_code_point_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const replacement = String.fromCharCode(0xFFFD);

            location.href = 'https://\u00E9xample.com/';
            const afterIdna = [
              location.href,
              navigation.currentEntry.url,
              history.length
            ].join(',');

            location.href = 'https://\uFF21example.com/?a=%zz&b=%E0%A4&c=%C3%28';
            const parsedAfterHref = new URL(location.href);
            const afterHref = [
              location.href,
              location.host,
              location.search,
              parsedAfterHref.searchParams.get('a'),
              parsedAfterHref.searchParams.get('b'),
              parsedAfterHref.searchParams.get('c'),
              parsedAfterHref.searchParams.toString(),
              navigation.currentEntry.url,
              history.length
            ].join(',');

            history.pushState({ step: 1 }, '', '?b=%E0%A4&a=%zz&a=1');
            const parsedAfterPush = new URL(location.href);
            const afterPush = [
              location.href,
              location.search,
              parsedAfterPush.searchParams.getAll('a').join(':'),
              parsedAfterPush.searchParams.get('b'),
              parsedAfterPush.searchParams.toString(),
              navigation.currentEntry.url,
              history.length
            ].join(',');

            const afterReplace = (() => {
              const mutated = new URL(location.href);
              mutated.searchParams.sort();
              mutated.searchParams.set('a', '%zz');
              history.replaceState({ step: 2 }, '', mutated.href);
              return [
                location.href,
                location.search,
                mutated.searchParams.getAll('a').join(':'),
                mutated.searchParams.get('b'),
                mutated.searchParams.toString(),
                navigation.currentEntry.url,
                history.length
              ].join(',');
            })();

            const invalidReplace = (() => {
              try {
                history.replaceState({ step: 3 }, '', 'https://%00example.com/');
                return 'false';
              } catch (err) {
                return [
                  String(err).includes('Invalid URL'),
                  location.href,
                  navigation.currentEntry.url,
                  history.length
                ].join(',');
              }
            })();

            document.getElementById('result').textContent = [
              afterIdna,
              afterHref,
              afterPush,
              afterReplace,
              invalidReplace
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/start", html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "https://xn--xample-9ua.com/,https://xn--xample-9ua.com/,2|https://aexample.com/?a=%zz&b=%E0%A4&c=%C3%28,aexample.com,?a=%zz&b=%E0%A4&c=%C3%28,%zz,\u{FFFD},\u{FFFD}(,a=%25zz&b=%EF%BF%BD&c=%EF%BF%BD%28,https://aexample.com/?a=%zz&b=%E0%A4&c=%C3%28,3|https://aexample.com/?b=%E0%A4&a=%zz&a=1,?b=%E0%A4&a=%zz&a=1,%zz:1,\u{FFFD},b=%EF%BF%BD&a=%25zz&a=1,https://aexample.com/?b=%E0%A4&a=%zz&a=1,4|https://aexample.com/?a=%25zz&b=%EF%BF%BD,?a=%25zz&b=%EF%BF%BD,%zz,\u{FFFD},a=%25zz&b=%EF%BF%BD,https://aexample.com/?a=%25zz&b=%EF%BF%BD,4|true,https://aexample.com/?a=%25zz&b=%EF%BF%BD,https://aexample.com/?a=%25zz&b=%EF%BF%BD,4",
    )?;
    assert_eq!(
        h.take_location_navigations(),
        vec![
            LocationNavigation {
                kind: LocationNavigationKind::HrefSet,
                from: "https://app.local/start".to_string(),
                to: "https://xn--xample-9ua.com/".to_string(),
            },
            LocationNavigation {
                kind: LocationNavigationKind::HrefSet,
                from: "https://xn--xample-9ua.com/".to_string(),
                to: "https://aexample.com/?a=%zz&b=%E0%A4&c=%C3%28".to_string(),
            },
        ]
    );
    Ok(())
}

#[test]
fn location_and_history_idna_invalid_label_residuals_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            location.href = 'https://example\u3002com./path';
            const afterDot = [
              location.href,
              navigation.currentEntry.url,
              history.length
            ].join(',');

            history.pushState({ step: 1 }, '', 'https://\u05D0.com/');
            const afterPush = [
              location.href,
              navigation.currentEntry.url,
              history.length
            ].join(',');

            const invalidHref = (() => {
              try {
                location.href = 'https://xn--/';
                return 'false';
              } catch (err) {
                return [
                  String(err).includes('Invalid URL'),
                  location.href,
                  navigation.currentEntry.url,
                  history.length
                ].join(',');
              }
            })();

            const invalidReplace = (() => {
              try {
                history.replaceState({ step: 2 }, '', 'https://a\u200Db.com/');
                return 'false';
              } catch (err) {
                return [
                  String(err).includes('Invalid URL'),
                  location.href,
                  navigation.currentEntry.url,
                  history.length
                ].join(',');
              }
            })();

            document.getElementById('result').textContent = [
              afterDot,
              afterPush,
              invalidHref,
              invalidReplace
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/start", html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "https://example.com./path,https://example.com./path,2|https://xn--4db.com/,https://xn--4db.com/,3|true,https://xn--4db.com/,https://xn--4db.com/,3|true,https://xn--4db.com/,https://xn--4db.com/,3",
    )?;
    assert_eq!(
        h.take_location_navigations(),
        vec![LocationNavigation {
            kind: LocationNavigationKind::HrefSet,
            from: "https://app.local/start".to_string(),
            to: "https://example.com./path".to_string(),
        },]
    );
    Ok(())
}

#[test]
fn location_assign_replace_reload_and_navigation_logs_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            location.assign('https://app.local/a?x=1#h');
            location.replace('/b');
            location.reload();
            document.getElementById('result').textContent = location.href;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "https://app.local/b")?;
    assert_eq!(h.location_reload_count(), 1);

    assert_eq!(
        h.take_location_navigations(),
        vec![
            LocationNavigation {
                kind: LocationNavigationKind::Assign,
                from: "about:blank".to_string(),
                to: "https://app.local/a?x=1#h".to_string(),
            },
            LocationNavigation {
                kind: LocationNavigationKind::Replace,
                from: "https://app.local/a?x=1#h".to_string(),
                to: "https://app.local/b".to_string(),
            },
            LocationNavigation {
                kind: LocationNavigationKind::Reload,
                from: "https://app.local/b".to_string(),
                to: "https://app.local/b".to_string(),
            },
        ]
    );
    assert!(h.take_location_navigations().is_empty());
    Ok(())
}

#[test]
fn location_mock_pages_load_on_navigation_and_reload() -> Result<()> {
    let html = r#"
        <button id='go'>go</button>
        <script>
          document.getElementById('go').addEventListener('click', () => {
            location.assign('https://app.local/next');
          });
        </script>
        "#;

    let first_mock = r#"
        <button id='reload'>reload</button>
        <p id='marker'>first</p>
        <script>
          document.getElementById('reload').addEventListener('click', () => {
            location.reload();
          });
        </script>
        "#;
    let second_mock = "<p id='marker'>second</p>";

    let mut h = Harness::from_html(html)?;
    h.set_location_mock_page("https://app.local/next", first_mock);
    h.click("#go")?;
    h.assert_text("#marker", "first")?;

    h.set_location_mock_page("https://app.local/next", second_mock);
    h.click("#reload")?;
    h.assert_text("#marker", "second")?;
    assert_eq!(h.location_reload_count(), 1);

    assert_eq!(
        h.take_location_navigations(),
        vec![
            LocationNavigation {
                kind: LocationNavigationKind::Assign,
                from: "about:blank".to_string(),
                to: "https://app.local/next".to_string(),
            },
            LocationNavigation {
                kind: LocationNavigationKind::Reload,
                from: "https://app.local/next".to_string(),
                to: "https://app.local/next".to_string(),
            },
        ]
    );
    Ok(())
}

#[test]
fn document_visibilitychange_fires_before_mock_page_navigation() -> Result<()> {
    let html = r#"
        <button id='go'>go</button>
        <script>
          localStorage.setItem('vis-log', '');
          document.addEventListener('visibilitychange', (event) => {
            localStorage.setItem(
              'vis-log',
              document.visibilityState + ':' +
              document.hidden + ':' +
              event.type + ':' +
              event.cancelable
            );
          });
          document.getElementById('go').addEventListener('click', () => {
            location.assign('https://app.local/next');
          });
        </script>
        "#;

    let next_mock = r#"
        <p id='result'></p>
        <script>
          document.getElementById('result').textContent =
            (localStorage.getItem('vis-log') || 'none') + '|' +
            document.visibilityState + ':' + document.hidden;
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.set_location_mock_page("https://app.local/next", next_mock);
    h.click("#go")?;
    h.assert_text(
        "#result",
        "hidden:true:visibilitychange:false|visible:false",
    )?;
    Ok(())
}

#[test]
fn document_onvisibilitychange_property_fires_before_mock_page_navigation() -> Result<()> {
    let html = r#"
        <button id='go'>go</button>
        <script>
          localStorage.setItem('vis-on', '');
          document.onvisibilitychange = (event) => {
            localStorage.setItem('vis-on', event.type + ':' + event.cancelable);
          };
          document.getElementById('go').addEventListener('click', () => {
            location.assign('https://app.local/next');
          });
        </script>
        "#;

    let next_mock = r#"
        <p id='result'></p>
        <script>
          document.getElementById('result').textContent = localStorage.getItem('vis-on') || 'none';
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.set_location_mock_page("https://app.local/next", next_mock);
    h.click("#go")?;
    h.assert_text("#result", "visibilitychange:false")?;
    Ok(())
}

#[test]
fn document_dom_content_loaded_fires_after_initial_scripts() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const log = [];
          log.push('start:' + document.readyState);
          document.addEventListener('DOMContentLoaded', (event) => {
            log.push('event:' + event.type + ':' + event.cancelable + ':' + document.readyState);
            document.getElementById('result').textContent = log.join('|');
          });
          log.push('end:' + document.readyState);
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text(
        "#result",
        "start:loading|end:loading|event:DOMContentLoaded:false:interactive",
    )?;
    Ok(())
}

#[test]
fn document_dom_content_loaded_ready_state_guard_pattern_works() -> Result<()> {
    let html = r#"
        <button id='late'>late</button>
        <p id='result'></p>
        <script>
          let setupCalls = 0;
          function setup() {
            setupCalls = setupCalls + 1;
          }

          if (document.readyState === 'loading') {
            document.addEventListener('DOMContentLoaded', setup);
          } else {
            setup();
          }

          document.addEventListener('DOMContentLoaded', () => {
            document.getElementById('result').textContent =
              'init:' + setupCalls + ':' + document.readyState;
          });

          document.getElementById('late').addEventListener('click', () => {
            let lateCalls = 0;
            function lateSetup() {
              lateCalls = lateCalls + 1;
            }

            if (document.readyState === 'loading') {
              document.addEventListener('DOMContentLoaded', lateSetup);
            } else {
              lateSetup();
            }

            document.getElementById('result').textContent =
              'late:' + setupCalls + ':' + lateCalls + ':' + document.readyState;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.assert_text("#result", "init:1:interactive")?;
    h.click("#late")?;
    h.assert_text("#result", "late:1:1:complete")?;
    Ok(())
}

#[test]
fn document_on_dom_content_loaded_property_does_not_fire() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          let addCalls = 0;
          let propCalls = 0;

          document.ondomcontentloaded = () => {
            propCalls = propCalls + 1;
          };

          document.addEventListener('DOMContentLoaded', () => {
            addCalls = addCalls + 1;
            document.getElementById('result').textContent =
              'add:' + addCalls + ':prop:' + propCalls;
          });
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "add:1:prop:0")?;
    Ok(())
}

#[test]
fn document_dom_content_loaded_fires_for_mock_page_navigation() -> Result<()> {
    let html = r#"
        <button id='go'>go</button>
        <script>
          document.getElementById('go').addEventListener('click', () => {
            location.assign('https://app.local/next');
          });
        </script>
        "#;

    let next_mock = r#"
        <p id='result'></p>
        <script>
          document.addEventListener('DOMContentLoaded', (event) => {
            document.getElementById('result').textContent =
              event.type + ':' + event.cancelable + ':' + document.readyState;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.set_location_mock_page("https://app.local/next", next_mock);
    h.click("#go")?;
    h.assert_text("#result", "DOMContentLoaded:false:interactive")?;
    Ok(())
}

#[test]
fn document_selectionchange_event_fires_when_text_selection_changes() -> Result<()> {
    let html = r#"
        <input id='field' value='hello'>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const field = document.getElementById('field');
          const logs = [];

          document.addEventListener('selectionchange', (event) => {
            logs.push(
              event.type + ':' +
              event.cancelable + ':' +
              field.selectionStart + '-' + field.selectionEnd + ':' +
              field.selectionDirection
            );
          });

          document.getElementById('run').addEventListener('click', () => {
            field.setSelectionRange(1, 4, 'forward');
            field.setSelectionRange(1, 4, 'forward');
            field.selectionStart = 2;
            field.selectionEnd = 3;
            field.selectionDirection = 'backward';
            field.selectionDirection = 'backward';
            field.select();

            document.getElementById('result').textContent =
              logs.length + '|' + logs.join(',');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "5|selectionchange:false:1-4:forward,selectionchange:false:2-4:none,selectionchange:false:2-3:none,selectionchange:false:2-3:backward,selectionchange:false:0-5:none",
    )?;
    Ok(())
}

#[test]
fn document_onselectionchange_property_assignment_works() -> Result<()> {
    let html = r#"
        <input id='field' value='hello'>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          let calls = 0;
          document.onselectionchange = (event) => {
            calls = calls + 1;
            document.getElementById('result').textContent =
              calls + ':' + event.type + ':' + event.cancelable;
          };

          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('field').select();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "1:selectionchange:false")?;
    Ok(())
}

#[test]
fn hash_only_location_navigation_does_not_trigger_mock_page_swap() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'>alive</p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            location.href = 'https://app.local/path';
            location.hash = 'frag';
            document.getElementById('result').textContent =
              document.getElementById('result').textContent + ':' + location.href;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.set_location_mock_page("https://app.local/path#frag", "<p id='result'>swapped</p>");
    h.click("#run")?;
    h.assert_text("#result", "alive:https://app.local/path#frag")?;

    assert_eq!(
        h.take_location_navigations(),
        vec![
            LocationNavigation {
                kind: LocationNavigationKind::HrefSet,
                from: "about:blank".to_string(),
                to: "https://app.local/path".to_string(),
            },
            LocationNavigation {
                kind: LocationNavigationKind::HrefSet,
                from: "https://app.local/path".to_string(),
                to: "https://app.local/path#frag".to_string(),
            },
        ]
    );
    Ok(())
}

#[test]
fn anchor_properties_and_to_string_work() -> Result<()> {
    let html = r#"
        <a id='link' href='/docs/page?x=1#intro'>hello</a>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            location.href = 'https://example.com/base/index.html?doc=1#docfrag';
            document.getElementById('link').download = 'report.txt';
            document.getElementById('link').hreflang = 'ja';
            document.getElementById('link').ping = 'https://p1.test https://p2.test';
            document.getElementById('link').referrerPolicy = 'no-referrer';
            document.getElementById('link').rel = 'noopener noreferrer';
            document.getElementById('link').target = '_blank';
            document.getElementById('link').type = 'text/plain';
            document.getElementById('link').attributionSrc = 'https://attr.test/src';
            document.getElementById('link').interestForElement = 'panel';
            document.getElementById('link').charset = 'utf-8';
            document.getElementById('link').coords = '0,0,10,10';
            document.getElementById('link').rev = 'prev';
            document.getElementById('link').shape = 'rect';

            document.getElementById('result').textContent =
              document.getElementById('link').href + '|' +
              document.getElementById('link').protocol + '|' +
              document.getElementById('link').host + '|' +
              document.getElementById('link').hostname + '|' +
              document.getElementById('link').port + '|' +
              document.getElementById('link').pathname + '|' +
              document.getElementById('link').search + '|' +
              document.getElementById('link').hash + '|' +
              document.getElementById('link').origin + '|' +
              document.getElementById('link').download + '|' +
              document.getElementById('link').hreflang + '|' +
              document.getElementById('link').ping + '|' +
              document.getElementById('link').referrerPolicy + '|' +
              document.getElementById('link').rel + '|' +
              document.getElementById('link').relList.length + '|' +
              document.getElementById('link').target + '|' +
              document.getElementById('link').type + '|' +
              document.getElementById('link').attributionSrc + '|' +
              document.getElementById('link').interestForElement + '|' +
              document.getElementById('link').charset + '|' +
              document.getElementById('link').coords + '|' +
              document.getElementById('link').rev + '|' +
              document.getElementById('link').shape + '|' +
              document.getElementById('link').toString();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "https://example.com/docs/page?x=1#intro|https:|example.com|example.com||/docs/page|?x=1|#intro|https://example.com|report.txt|ja|https://p1.test https://p2.test|no-referrer|noopener noreferrer|2|_blank|text/plain|https://attr.test/src|panel|utf-8|0,0,10,10|prev|rect|https://example.com/docs/page?x=1#intro",
    )?;
    Ok(())
}

#[test]
fn anchor_username_password_and_url_part_setters_work() -> Result<()> {
    let html = r#"
        <a id='cred' href='https://u:p@example.com:8443/p?q=1#h'>cred</a>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const initial =
              document.getElementById('cred').username + ':' +
              document.getElementById('cred').password + ':' +
              document.getElementById('cred').host + ':' +
              document.getElementById('cred').origin;

            document.getElementById('cred').username = 'alice';
            document.getElementById('cred').password = 'secret';
            document.getElementById('cred').protocol = 'http:';
            document.getElementById('cred').hostname = 'api.example.test';
            document.getElementById('cred').port = '9090';
            document.getElementById('cred').pathname = 'docs';
            document.getElementById('cred').search = 'k=v';
            document.getElementById('cred').hash = 'frag';

            document.getElementById('result').textContent =
              initial + '|' +
              document.getElementById('cred').href + '|' +
              document.getElementById('cred').username + '|' +
              document.getElementById('cred').password + '|' +
              document.getElementById('cred').protocol + '|' +
              document.getElementById('cred').host + '|' +
              document.getElementById('cred').pathname + '|' +
              document.getElementById('cred').search + '|' +
              document.getElementById('cred').hash + '|' +
              document.getElementById('cred').origin + '|' +
              document.getElementById('cred').toString();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "u:p:example.com:8443:https://example.com:8443|http://alice:secret@api.example.test:9090/docs?k=v#frag|alice|secret|http:|api.example.test:9090|/docs|?k=v|#frag|http://api.example.test:9090|http://alice:secret@api.example.test:9090/docs?k=v#frag",
    )?;
    Ok(())
}

#[test]
fn anchor_text_alias_and_read_only_properties_work() -> Result<()> {
    let html = r#"
        <a id='link' href='https://example.com/start' rel='noopener'>old</a>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('link').text = 'Updated text';

            let originReadOnly = 'no';
            try {
              document.getElementById('link').origin = 'https://evil.example';
            } catch (e) {
              originReadOnly = 'yes';
            }

            let relListReadOnly = 'no';
            try {
              document.getElementById('link').relList = 'x';
            } catch (e) {
              relListReadOnly = 'yes';
            }

            document.getElementById('result').textContent =
              document.getElementById('link').textContent + ':' +
              document.getElementById('link').text + ':' +
              originReadOnly + ':' +
              relListReadOnly + ':' +
              document.getElementById('link').origin + ':' +
              document.getElementById('link').relList.length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "Updated text:Updated text:yes:yes:https://example.com:1",
    )?;
    Ok(())
}

#[test]
fn anchor_click_follows_href_for_relative_and_non_http_urls() -> Result<()> {
    let html = r#"
        <a id='web' href='/docs/page?x=1#intro'>web</a>
        <a id='mail' href='mailto:m.bluth@example.com'>mail</a>
        <a id='phone' href='tel:+123456789'>phone</a>
        "#;

    let mut h = Harness::from_html_with_url("https://example.com/base/index.html", html)?;
    h.click("#web")?;
    h.click("#mail")?;
    h.click("#phone")?;

    assert_eq!(
        h.take_location_navigations(),
        vec![
            LocationNavigation {
                kind: LocationNavigationKind::Assign,
                from: "https://example.com/base/index.html".to_string(),
                to: "https://example.com/docs/page?x=1#intro".to_string(),
            },
            LocationNavigation {
                kind: LocationNavigationKind::Assign,
                from: "https://example.com/docs/page?x=1#intro".to_string(),
                to: "mailto:m.bluth@example.com".to_string(),
            },
            LocationNavigation {
                kind: LocationNavigationKind::Assign,
                from: "mailto:m.bluth@example.com".to_string(),
                to: "tel:+123456789".to_string(),
            },
        ]
    );
    Ok(())
}

#[test]
fn anchor_click_navigation_is_skipped_without_href_download_or_target_blank() -> Result<()> {
    let html = r#"
        <a id='nohref'>nohref</a>
        <a id='blank' href='/blank' target='_blank'>blank</a>
        <a id='download' href='/report.csv' download='report.csv'>download</a>
        <a id='self' href='/self'>self</a>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/start", html)?;
    h.click("#nohref")?;
    h.click("#blank")?;
    h.click("#download")?;
    h.click("#self")?;

    assert_eq!(
        h.take_location_navigations(),
        vec![LocationNavigation {
            kind: LocationNavigationKind::Assign,
            from: "https://app.local/start".to_string(),
            to: "https://app.local/self".to_string(),
        }]
    );
    Ok(())
}

#[test]
fn anchor_invalid_and_special_host_click_matrix_work() -> Result<()> {
    let html = r#"
        <a id='bad' href='http://'>bad</a>
        <a id='bad-query' href='http:?x'>bad query</a>
        <a id='blank' href='https://example.com:abc/report' target='_blank'>blank</a>
        <a id='download' href='https://example.com:abc/report' download='report.csv'>download</a>
        <a id='hostless' href='http:\\Example.COM\\docs\\page?x=1#frag'>hostless</a>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/start", html)?;
    h.click("#bad")?;
    h.click("#bad-query")?;
    h.click("#blank")?;
    h.click("#download")?;
    h.click("#hostless")?;

    assert_eq!(
        h.take_location_navigations(),
        vec![LocationNavigation {
            kind: LocationNavigationKind::Assign,
            from: "https://app.local/start".to_string(),
            to: "http://example.com/docs/page?x=1#frag".to_string(),
        }]
    );
    Ok(())
}

#[test]
fn press_enter_activates_anchor_with_href_and_respects_keydown_prevent_default() -> Result<()> {
    let html = r#"
        <a id='go' href='/go'>Go</a>
        <a id='blocked' href='/blocked'>Blocked</a>
        <a id='plain'>Plain</a>
        <p id='result'></p>
        <script>
          let clicks = 0;
          document.getElementById('go').addEventListener('click', () => {
            clicks = clicks + 1;
            document.getElementById('result').textContent = 'go:' + clicks;
          });
          document.getElementById('blocked').addEventListener('keydown', (event) => {
            event.preventDefault();
          });
          document.getElementById('blocked').addEventListener('click', () => {
            document.getElementById('result').textContent = 'blocked-click';
          });
          document.getElementById('plain').addEventListener('click', () => {
            document.getElementById('result').textContent = 'plain-click';
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/start", html)?;
    h.press_enter("#go")?;
    h.assert_text("#result", "go:1")?;
    h.press_enter("#blocked")?;
    h.assert_text("#result", "go:1")?;
    h.press_enter("#plain")?;
    h.assert_text("#result", "go:1")?;

    assert_eq!(
        h.take_location_navigations(),
        vec![LocationNavigation {
            kind: LocationNavigationKind::Assign,
            from: "https://app.local/start".to_string(),
            to: "https://app.local/go".to_string(),
        }]
    );
    Ok(())
}

#[test]
fn history_properties_push_state_and_replace_state_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const initialLen = history.length;
            const initialState = history.state === null ? 'null' : 'non-null';
            history.pushState({ step: 1 }, '', 'https://app.local/one');
            const pushed = history.length + ':' + history.state.step + ':' + location.href;
            history.replaceState({ step: 2 }, '', 'https://app.local/two');
            const replaced = history.length + ':' + history.state.step + ':' + location.href;
            document.getElementById('result').textContent =
              initialLen + ':' + initialState + '|' + pushed + '|' + replaced + '|' + window.history.scrollRestoration;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "1:null|2:1:https://app.local/one|2:2:https://app.local/two|auto",
    )?;
    Ok(())
}

#[test]
fn history_back_forward_and_go_dispatch_popstate_with_state() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          window.addEventListener('popstate', (event) => {
            document.getElementById('result').textContent =
              document.getElementById('result').textContent +
              '[' + (event.state === null ? 'null' : event.state) + '@' + location.href + ']';
          });

          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('result').textContent = '';
            history.pushState('A', '', 'https://app.local/a');
            history.pushState('B', '', 'https://app.local/b');
            history.back();
            history.forward();
            history.go(-2);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "[A@https://app.local/a][B@https://app.local/b][null@about:blank]",
    )?;
    Ok(())
}

#[test]
fn history_out_of_bounds_navigation_is_noop() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            history.pushState('A', '', 'https://app.local/a');
            history.go(10);
            history.forward();
            history.go(-10);
            document.getElementById('result').textContent =
              history.length + ':' + history.state + ':' + location.href;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "2:A:https://app.local/a")?;
    Ok(())
}

#[test]
fn history_go_reload_works_with_location_mock_page() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            history.go();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.set_location_mock_page("about:blank", "<p id='marker'>reloaded</p>");
    h.click("#run")?;
    h.assert_text("#marker", "reloaded")?;
    assert_eq!(h.location_reload_count(), 1);
    Ok(())
}

#[test]
fn history_scroll_restoration_setter_and_window_history_access_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const before = window.history.scrollRestoration;
            history.scrollRestoration = 'manual';
            document.getElementById('result').textContent =
              before + ':' + history.scrollRestoration + ':' + window.history.length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "auto:manual:1")?;
    Ok(())
}

#[test]
fn history_read_only_and_invalid_scroll_restoration_are_rejected() {
    let readonly_err = Harness::from_html(
        r#"
        <script>
          window.history.length = 2;
        </script>
        "#,
    )
    .expect_err("history.length should be read-only");
    match readonly_err {
        Error::ScriptRuntime(msg) => assert_eq!(msg, "history.length is read-only"),
        other => panic!("unexpected error: {other:?}"),
    }

    let invalid_mode_err = Harness::from_html(
        r#"
        <script>
          history.scrollRestoration = 'smooth';
        </script>
        "#,
    )
    .expect_err("invalid scrollRestoration value should fail");
    match invalid_mode_err {
        Error::ScriptRuntime(msg) => {
            assert_eq!(msg, "history.scrollRestoration must be 'auto' or 'manual'")
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn document_has_focus_reports_active_element_state() -> Result<()> {
    let html = r#"
        <input id='name'>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const before = document.hasFocus();
            document.getElementById('name').focus();
            const during = document.hasFocus();
            document.getElementById('name').blur();
            const after = document.hasFocus();
            document.getElementById('result').textContent = before + ':' + during + ':' + after;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "false:true:false")?;
    Ok(())
}

#[test]
fn document_body_chain_supports_query_selector_and_query_selector_all() -> Result<()> {
    let html = r#"
        <body>
          <div id='a' class='item'></div>
          <div id='b' class='item'></div>
          <button id='btn'>run</button>
          <p id='result'></p>
          <script>
            document.getElementById('btn').addEventListener('click', () => {
              const picked = document.body.querySelector('.item');
              const total = document.body.querySelectorAll('.item').length;
              picked.classList.remove('item');
              document.getElementById('result').textContent =
                picked.id + ':' + total + ':' + document.body.querySelectorAll('.item').length;
            });
          </script>
        </body>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "a:2:1")?;
    Ok(())
}

#[test]
fn class_list_for_each_supports_single_arg_and_index() -> Result<()> {
    let html = r#"
        <div id='box' class='red green blue'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let joined = '';
            let indexes = '';
            document.getElementById('box').classList.forEach((name, index) => {
              joined = joined + name;
              indexes = indexes + index;
            });
            document.getElementById('result').textContent = joined + ':' + indexes;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "redgreenblue:012")?;
    Ok(())
}

#[test]
fn element_click_method_from_script_works() -> Result<()> {
    let html = r#"
        <button id='trigger'>click me</button>
        <input id='agree' type='checkbox'>
        <p id='result'></p>
        <script>
          document.getElementById('trigger').addEventListener('click', () => {
            document.getElementById('agree').click();
            document.getElementById('result').textContent =
              (document.getElementById('agree').checked ? 'checked' : 'unchecked');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#trigger")?;
    h.assert_text("#result", "checked")?;
    h.click("#trigger")?;
    h.assert_text("#result", "unchecked")?;
    Ok(())
}

#[test]
fn anchor_download_blob_click_inside_handler_keeps_dom_state_for_following_statements() -> Result<()>
{
    let html = r#"
        <html><body>
          <button id='run'>run</button>
          <div id='result'></div>
          <script>
            document.getElementById('run').addEventListener('click', () => {
              const blob = new Blob(['abc'], { type: 'text/plain' });
              const url = URL.createObjectURL(blob);
              const a = document.createElement('a');
              a.href = url;
              a.download = 'test.csv';
              document.body.appendChild(a);

              document.getElementById('result').textContent = 'before';
              a.click();
              document.getElementById('result').textContent += '|after';

              a.remove();
              URL.revokeObjectURL(url);
            });
          </script>
        </body></html>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "before|after")?;
    assert!(h.take_location_navigations().is_empty());
    assert_eq!(
        h.take_downloads(),
        vec![DownloadArtifact {
            filename: Some("test.csv".to_string()),
            mime_type: Some("text/plain".to_string()),
            bytes: b"abc".to_vec(),
        }]
    );
    Ok(())
}

#[test]
fn element_scroll_into_view_method_from_script_works() -> Result<()> {
    let html = r#"
        <button id='trigger'>scroll target</button>
        <section id='target'></section>
        <p id='result'></p>
        <script>
          document.getElementById('trigger').addEventListener('click', () => {
            document.getElementById('target').scrollIntoView();
            document.getElementById('result').textContent = 'done';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#trigger")?;
    h.assert_text("#result", "done")?;
    Ok(())
}

#[test]
fn element_scroll_into_view_accepts_optional_argument() -> Result<()> {
    let html = r#"
        <button id='trigger'>target</button>
        <p id='result'></p>
        <script>
          document.getElementById('trigger').addEventListener('click', () => {
            document.getElementById('trigger').scrollIntoView({ behavior: 'smooth', block: 'start' });
            document.getElementById('result').textContent = 'ok';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#trigger")?;
    h.assert_text("#result", "ok")?;
    Ok(())
}

#[test]
fn element_scroll_into_view_accepts_boolean_argument_in_expression_context() -> Result<()> {
    let html = r#"
        <button id='trigger'>target</button>
        <p id='result'></p>
        <script>
          document.getElementById('trigger').addEventListener('click', () => {
            const ret = document.getElementById('trigger').scrollIntoView(false);
            document.getElementById('result').textContent = String(ret === undefined);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#trigger")?;
    h.assert_text("#result", "true")?;
    Ok(())
}

#[test]
fn element_scroll_into_view_accepts_options_argument_in_expression_context() -> Result<()> {
    let html = r#"
        <button id='trigger'>target</button>
        <p id='result'></p>
        <script>
          document.getElementById('trigger').addEventListener('click', () => {
            const ret = document.getElementById('trigger').scrollIntoView({
              behavior: 'smooth',
              block: 'end',
              inline: 'nearest',
              container: 'nearest'
            });
            document.getElementById('result').textContent = String(ret === undefined);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#trigger")?;
    h.assert_text("#result", "true")?;
    Ok(())
}

#[test]
fn element_animate_returns_animation_object_and_respects_id_option() -> Result<()> {
    let html = r#"
        <button id='trigger'>run</button>
        <div id='box'></div>
        <p id='result'></p>
        <script>
          document.getElementById('trigger').addEventListener('click', () => {
            const animation = document.getElementById('box').animate(
              [
                { transform: 'rotate(0deg) scale(1)' },
                { transform: 'rotate(360deg) scale(0)' }
              ],
              { duration: 2000, iterations: 1, id: 'spin' }
            );
            document.getElementById('result').textContent =
              typeof animation + ':' +
              animation.id + ':' +
              animation.playState + ':' +
              String(animation.currentTime) + ':' +
              String(typeof animation.play === 'function') + ':' +
              String(String(animation) === '[object Animation]');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#trigger")?;
    h.assert_text("#result", "object:spin:running:0:true:true")?;
    Ok(())
}

#[test]
fn element_animate_accepts_numeric_options_argument() -> Result<()> {
    let html = r#"
        <button id='trigger'>run</button>
        <div id='box'></div>
        <p id='result'></p>
        <script>
          document.getElementById('trigger').addEventListener('click', () => {
            const animation = document.getElementById('box').animate(
              { transform: 'translateX(300px)' },
              1000
            );
            document.getElementById('result').textContent =
              String(animation.options) + ':' +
              String(animation.id === '') + ':' +
              animation.playState + ':' +
              String(animation.timeline === null);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#trigger")?;
    h.assert_text("#result", "1000:true:running:true")?;
    Ok(())
}

#[test]
fn element_animate_applies_timeline_and_range_options() -> Result<()> {
    let html = r#"
        <button id='trigger'>run</button>
        <div id='box'></div>
        <p id='result'></p>
        <script>
          document.getElementById('trigger').addEventListener('click', () => {
            const timeline = { kind: 'view' };
            const animation = document.getElementById('box').animate(
              { opacity: [0, 1], transform: ['scaleX(0)', 'scaleX(1)'] },
              {
                fill: 'both',
                duration: 1,
                timeline,
                rangeStart: 'cover 0%',
                rangeEnd: 'cover 100%'
              }
            );
            document.getElementById('result').textContent =
              String(animation.timeline === timeline) + ':' +
              animation.rangeStart + ':' +
              animation.rangeEnd;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#trigger")?;
    h.assert_text("#result", "true:cover 0%:cover 100%")?;
    Ok(())
}

#[test]
fn element_animate_rejects_zero_arguments() -> Result<()> {
    let html = r#"
        <button id='trigger'>run</button>
        <div id='box'></div>
        <script>
          document.getElementById('trigger').addEventListener('click', () => {
            document.getElementById('box').animate();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#trigger") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("animate requires one or two arguments"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}

#[test]
fn element_scroll_by_accepts_xy_arguments_and_returns_undefined() -> Result<()> {
    let html = r#"
        <button id='trigger'>run</button>
        <div id='target'></div>
        <p id='result'></p>
        <script>
          const out = document.getElementById('result');
          document.addEventListener('scroll', () => {
            out.textContent = out.textContent + 's';
          });
          document.addEventListener('scrollend', () => {
            out.textContent = out.textContent + 'e';
          });
          document.getElementById('trigger').addEventListener('click', () => {
            const target = document.getElementById('target');
            const first = target.scrollBy(300, 300);
            const second = target.scrollBy(-50, 20);
            out.textContent = String(first === undefined && second === undefined) + ':' + out.textContent;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#trigger")?;
    h.assert_text("#result", "true:sese")?;
    Ok(())
}

#[test]
fn element_scroll_by_accepts_options_argument() -> Result<()> {
    let html = r#"
        <button id='trigger'>run</button>
        <div id='target'></div>
        <p id='result'></p>
        <script>
          const out = document.getElementById('result');
          document.addEventListener('scroll', () => {
            out.textContent = out.textContent + 's';
          });
          document.addEventListener('scrollend', () => {
            out.textContent = out.textContent + 'e';
          });
          document.getElementById('trigger').addEventListener('click', () => {
            const ret = document.getElementById('target').scrollBy({
              top: 100,
              left: 100,
              behavior: 'smooth'
            });
            out.textContent = String(ret === undefined) + ':' + out.textContent;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#trigger")?;
    h.assert_text("#result", "true:se")?;
    Ok(())
}

#[test]
fn element_scroll_by_rejects_more_than_two_arguments() -> Result<()> {
    let html = r#"
        <button id='trigger'>run</button>
        <div id='target'></div>
        <script>
          document.getElementById('trigger').addEventListener('click', () => {
            document.getElementById('target').scrollBy(1, 2, 3);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#trigger") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("supports zero, one, or two arguments"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}

#[test]
fn element_scroll_to_accepts_xy_arguments_and_returns_undefined() -> Result<()> {
    let html = r#"
        <button id='trigger'>run</button>
        <div id='target'></div>
        <p id='result'></p>
        <script>
          const out = document.getElementById('result');
          document.addEventListener('scroll', () => {
            out.textContent = out.textContent + 's';
          });
          document.addEventListener('scrollend', () => {
            out.textContent = out.textContent + 'e';
          });
          document.getElementById('trigger').addEventListener('click', () => {
            const target = document.getElementById('target');
            const first = target.scrollTo(0, 1000);
            const second = target.scrollTo(10, 20);
            out.textContent = String(first === undefined && second === undefined) + ':' + out.textContent;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#trigger")?;
    h.assert_text("#result", "true:sese")?;
    Ok(())
}

#[test]
fn element_scroll_to_accepts_options_argument() -> Result<()> {
    let html = r#"
        <button id='trigger'>run</button>
        <div id='target'></div>
        <p id='result'></p>
        <script>
          const out = document.getElementById('result');
          document.addEventListener('scroll', () => {
            out.textContent = out.textContent + 's';
          });
          document.addEventListener('scrollend', () => {
            out.textContent = out.textContent + 'e';
          });
          document.getElementById('trigger').addEventListener('click', () => {
            const ret = document.getElementById('target').scrollTo({
              top: 100,
              left: 100,
              behavior: 'smooth'
            });
            out.textContent = String(ret === undefined) + ':' + out.textContent;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#trigger")?;
    h.assert_text("#result", "true:se")?;
    Ok(())
}

#[test]
fn element_scroll_to_is_alias_for_scroll() -> Result<()> {
    let html = r#"
        <button id='trigger'>run</button>
        <div id='target'></div>
        <p id='result'></p>
        <script>
          const out = document.getElementById('result');
          document.addEventListener('scroll', () => {
            out.textContent = out.textContent + 's';
          });
          document.addEventListener('scrollend', () => {
            out.textContent = out.textContent + 'e';
          });
          document.getElementById('trigger').addEventListener('click', () => {
            const target = document.getElementById('target');
            const fromScroll = target.scroll({ top: 55, left: 44, behavior: 'auto' });
            const fromScrollTo = target.scrollTo({ top: 55, left: 44, behavior: 'auto' });
            out.textContent =
              String(fromScroll === undefined && fromScrollTo === undefined) + ':' + out.textContent;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#trigger")?;
    h.assert_text("#result", "true:ses")?;
    Ok(())
}

#[test]
fn element_scroll_to_rejects_more_than_two_arguments() -> Result<()> {
    let html = r#"
        <button id='trigger'>run</button>
        <div id='target'></div>
        <script>
          document.getElementById('trigger').addEventListener('click', () => {
            document.getElementById('target').scrollTo(1, 2, 3);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#trigger") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("supports zero, one, or two arguments"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}

#[test]
fn element_scroll_accepts_xy_arguments_and_returns_undefined() -> Result<()> {
    let html = r#"
        <button id='trigger'>run</button>
        <div id='target'></div>
        <p id='result'></p>
        <script>
          const out = document.getElementById('result');
          document.addEventListener('scroll', () => {
            out.textContent = out.textContent + 's';
          });
          document.addEventListener('scrollend', () => {
            out.textContent = out.textContent + 'e';
          });
          document.getElementById('trigger').addEventListener('click', () => {
            const target = document.getElementById('target');
            const first = target.scroll(0, 1000);
            const second = target.scroll(10, 20);
            out.textContent = String(first === undefined && second === undefined) + ':' + out.textContent;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#trigger")?;
    h.assert_text("#result", "true:sese")?;
    Ok(())
}

#[test]
fn element_scroll_accepts_options_argument() -> Result<()> {
    let html = r#"
        <button id='trigger'>run</button>
        <div id='target'></div>
        <p id='result'></p>
        <script>
          const out = document.getElementById('result');
          document.addEventListener('scroll', () => {
            out.textContent = out.textContent + 's';
          });
          document.addEventListener('scrollend', () => {
            out.textContent = out.textContent + 'e';
          });
          document.getElementById('trigger').addEventListener('click', () => {
            const ret = document.getElementById('target').scroll({
              top: 100,
              left: 100,
              behavior: 'smooth'
            });
            out.textContent = String(ret === undefined) + ':' + out.textContent;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#trigger")?;
    h.assert_text("#result", "true:se")?;
    Ok(())
}

#[test]
fn element_scroll_is_alias_for_scroll_to() -> Result<()> {
    let html = r#"
        <button id='trigger'>run</button>
        <div id='target'></div>
        <p id='result'></p>
        <script>
          const out = document.getElementById('result');
          document.addEventListener('scroll', () => {
            out.textContent = out.textContent + 's';
          });
          document.addEventListener('scrollend', () => {
            out.textContent = out.textContent + 'e';
          });
          document.getElementById('trigger').addEventListener('click', () => {
            const target = document.getElementById('target');
            const fromScrollTo = target.scrollTo({ top: 55, left: 44, behavior: 'auto' });
            const fromScroll = target.scroll({ top: 55, left: 44, behavior: 'auto' });
            out.textContent =
              String(fromScrollTo === undefined && fromScroll === undefined) + ':' + out.textContent;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#trigger")?;
    h.assert_text("#result", "true:ses")?;
    Ok(())
}

#[test]
fn element_scroll_rejects_more_than_two_arguments() -> Result<()> {
    let html = r#"
        <button id='trigger'>run</button>
        <div id='target'></div>
        <script>
          document.getElementById('trigger').addEventListener('click', () => {
            document.getElementById('target').scroll(1, 2, 3);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#trigger") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("supports zero, one, or two arguments"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}

#[test]
fn add_event_listener_accepts_async_arrow_callback() -> Result<()> {
    let html = r#"
        <button id='trigger'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('trigger').addEventListener('click', async () => {
            document.getElementById('result').textContent = 'ok';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#trigger")?;
    h.assert_text("#result", "ok")?;
    Ok(())
}

#[test]
fn form_submit_method_bypasses_submit_event_and_validation() -> Result<()> {
    let html = r#"
        <dialog id='dialog'>
          <form id='f' method='dialog'>
            <input id='name' required>
          </form>
        </dialog>
        <button id='trigger'>run</button>
        <p id='result'></p>
        <script>
          const dialog = document.getElementById('dialog');
          const form = document.getElementById('f');
          let marker = 'none';
          document.getElementById('f').addEventListener('submit', (event) => {
            marker = 'submitted';
          });
          document.getElementById('trigger').addEventListener('click', () => {
            dialog.showModal();
            form.submit();
            document.getElementById('result').textContent = marker + ':' + dialog.open;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#trigger")?;
    h.assert_text("#result", "none:false")?;
    Ok(())
}

#[test]
fn harness_submit_runs_validation_and_submit_event() -> Result<()> {
    let html = r#"
        <form id='f'>
          <input id='name' required>
        </form>
        <p id='result'>none</p>
        <script>
          document.getElementById('f').addEventListener('submit', () => {
            document.getElementById('result').textContent = 'submitted';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.submit("#f")?;
    h.assert_text("#result", "none")?;
    h.type_text("#name", "ok")?;
    h.submit("#f")?;
    h.assert_text("#result", "submitted")?;
    Ok(())
}

#[test]
fn form_request_submit_runs_validation_and_submit_event() -> Result<()> {
    let html = r#"
        <form id='f'>
          <input id='name' required>
        </form>
        <button id='empty'>empty</button>
        <button id='filled'>filled</button>
        <p id='result'>none</p>
        <script>
          const form = document.getElementById('f');
          const name = document.getElementById('name');
          let marker = 'none';
          form.addEventListener('submit', (event) => {
            event.preventDefault();
            marker = 'submitted';
          });
          document.getElementById('empty').addEventListener('click', () => {
            form.requestSubmit();
            document.getElementById('result').textContent = marker;
          });
          document.getElementById('filled').addEventListener('click', () => {
            name.value = 'ok';
            form.requestSubmit();
            document.getElementById('result').textContent = marker;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#empty")?;
    h.assert_text("#result", "none")?;
    h.click("#filled")?;
    h.assert_text("#result", "submitted")?;
    Ok(())
}

#[test]
fn form_request_submit_accepts_submitter_argument() -> Result<()> {
    let html = r#"
        <form id='f'>
          <input id='name' required value='ok'>
          <button id='submitter' type='submit'>send</button>
        </form>
        <button id='trigger'>run</button>
        <p id='result'>none</p>
        <script>
          const form = document.getElementById('f');
          const submitter = document.getElementById('submitter');
          form.addEventListener('submit', (event) => {
            event.preventDefault();
            document.getElementById('result').textContent = 'submitted';
          });
          document.getElementById('trigger').addEventListener('click', () => {
            form.requestSubmit(submitter);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#trigger")?;
    h.assert_text("#result", "submitted")?;
    Ok(())
}

#[test]
fn form_request_submit_accepts_image_submitter_argument() -> Result<()> {
    let html = r#"
        <form id='f'>
          <input id='name' required value='ok'>
          <input id='submitter' type='image' alt='send' src='/send.png'>
        </form>
        <button id='trigger'>run</button>
        <p id='result'>none</p>
        <script>
          const form = document.getElementById('f');
          const submitter = document.getElementById('submitter');
          form.addEventListener('submit', (event) => {
            event.preventDefault();
            document.getElementById('result').textContent = 'submitted';
          });
          document.getElementById('trigger').addEventListener('click', () => {
            form.requestSubmit(submitter);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#trigger")?;
    h.assert_text("#result", "submitted")?;
    Ok(())
}

#[test]
fn form_request_submit_rejects_non_submitter_argument() -> Result<()> {
    let html = r#"
        <form id='f'>
          <input id='name' required value='ok'>
          <input id='plain' type='text' value='x'>
        </form>
        <button id='trigger'>run</button>
        <script>
          const form = document.getElementById('f');
          const plain = document.getElementById('plain');
          document.getElementById('trigger').addEventListener('click', () => {
            form.requestSubmit(plain);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#trigger") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("requestSubmit submitter must be a submit control"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}

#[test]
fn form_request_submit_rejects_submitter_from_another_form() -> Result<()> {
    let html = r#"
        <form id='a'>
          <input id='name' required value='ok'>
          <button id='a-submit' type='submit'>a</button>
        </form>
        <form id='b'>
          <button id='b-submit' type='submit'>b</button>
        </form>
        <button id='trigger'>run</button>
        <script>
          const a = document.getElementById('a');
          const bSubmit = document.getElementById('b-submit');
          document.getElementById('trigger').addEventListener('click', () => {
            a.requestSubmit(bSubmit);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#trigger") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("requestSubmit submitter must belong to the target form"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}

#[test]
fn form_reset_method_dispatches_reset_and_restores_defaults() -> Result<()> {
    let html = r#"
        <form id='f'>
          <input id='name' value='default'>
          <input id='agree' type='checkbox' checked>
        </form>
        <button id='trigger'>run</button>
        <p id='result'></p>
        <script>
          let marker = '';
          document.getElementById('f').addEventListener('reset', () => {
            marker = marker + 'reset';
          });
          document.getElementById('trigger').addEventListener('click', () => {
            document.getElementById('name').value = 'changed';
            document.getElementById('agree').checked = false;
            document.getElementById('f').reset();
            document.getElementById('result').textContent =
              marker + ':' +
              document.getElementById('name').value + ':' +
              (document.getElementById('agree').checked ? 'on' : 'off');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#trigger")?;
    h.assert_text("#result", "reset:default:on")?;
    Ok(())
}

#[test]
fn dialog_show_modal_close_and_toggle_events_work() -> Result<()> {
    let html = r#"
        <dialog id='dialog'>
          <button id='close' type='button'>Close</button>
          <form method='dialog' id='form'>
            <p>
              <label for='fav-animal'>Favorite animal:</label>
              <select id='fav-animal' name='favAnimal' required>
                <option></option>
                <option>Brine shrimp</option>
                <option>Red panda</option>
                <option>Spider monkey</option>
              </select>
            </p>
            <button id='submit' type='submit'>Confirm</button>
          </form>
        </dialog>
        <button id='open'>Open dialog</button>
        <p id='result'></p>
        <script>
          const dialog = document.getElementById('dialog');
          let logs = '';

          dialog.addEventListener('beforetoggle', (event) => {
            logs = logs + 'before:' + event.oldState + '>' + event.newState + '|';
          });
          dialog.addEventListener('toggle', (event) => {
            logs = logs + 'toggle:' + event.newState + '|';
          });
          dialog.addEventListener('close', () => {
            logs = logs + 'close:' + dialog.returnValue + '|';
          });

          document.getElementById('open').addEventListener('click', () => {
            dialog.showModal();
            dialog.close('Red panda');
            document.getElementById('result').textContent = logs + 'open=' + dialog.open;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#open")?;
    h.assert_text(
        "#result",
        "before:closed>open|toggle:open|before:open>closed|toggle:closed|close:Red panda|open=false",
    )?;
    Ok(())
}

#[test]
fn dialog_request_close_fires_cancel_and_can_be_prevented() -> Result<()> {
    let html = r#"
        <dialog id='dialog' open></dialog>
        <button id='trigger'>run</button>
        <p id='result'></p>
        <script>
          const dialog = document.getElementById('dialog');
          let marker = '';
          dialog.addEventListener('cancel', (event) => {
            marker = marker + 'cancel:' + dialog.returnValue;
            dialog.returnValue = '';
            event.preventDefault();
          });
          dialog.addEventListener('close', () => {
            marker = marker + '|close';
          });
          document.getElementById('trigger').addEventListener('click', () => {
            dialog.returnValue = 'seed';
            dialog.requestClose('next');
            document.getElementById('result').textContent =
              marker + '|open=' + dialog.open + '|value=' + dialog.returnValue;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#trigger")?;
    h.assert_text("#result", "cancel:next|open=true|value=")?;
    Ok(())
}

#[test]
fn dialog_form_method_dialog_closes_and_keeps_submit_return_value() -> Result<()> {
    let html = r#"
        <dialog id='dialog'>
          <form id='form' method='dialog'>
            <select id='fav-animal' required>
              <option></option>
              <option>Brine shrimp</option>
              <option>Red panda</option>
              <option>Spider monkey</option>
            </select>
            <button id='submit' type='submit'>Confirm</button>
          </form>
        </dialog>
        <button id='trigger'>run</button>
        <p id='result'></p>
        <script>
          const dialog = document.getElementById('dialog');
          const form = document.getElementById('form');
          const select = document.getElementById('fav-animal');

          form.addEventListener('submit', () => {
            dialog.returnValue = select.value;
          });
          dialog.addEventListener('close', () => {
            document.getElementById('result').textContent =
              dialog.returnValue + ':' + dialog.open;
          });

          document.getElementById('trigger').addEventListener('click', () => {
            dialog.show();
            select.value = 'Spider monkey';
            document.getElementById('submit').click();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#trigger")?;
    h.assert_text("#result", "Spider monkey:false")?;
    Ok(())
}

#[test]
fn dialog_form_submit_is_blocked_when_required_control_is_empty() -> Result<()> {
    let html = r#"
        <dialog id='dialog'>
          <form id='form' method='dialog'>
            <select id='fav-animal' required>
              <option></option>
              <option>Brine shrimp</option>
            </select>
            <button id='submit' type='submit'>Confirm</button>
          </form>
        </dialog>
        <button id='trigger'>run</button>
        <p id='result'></p>
        <script>
          const dialog = document.getElementById('dialog');
          let marker = 'none';
          document.getElementById('form').addEventListener('submit', () => {
            marker = 'submitted';
          });
          dialog.addEventListener('close', () => {
            marker = marker + '|closed';
          });
          document.getElementById('trigger').addEventListener('click', () => {
            dialog.showModal();
            document.getElementById('submit').click();
            document.getElementById('result').textContent = marker + ':' + dialog.open;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#trigger")?;
    h.assert_text("#result", "none:true")?;
    Ok(())
}

#[test]
fn dialog_closed_by_property_reflects_closedby_attribute() -> Result<()> {
    let html = r#"
        <dialog id='dialog' closedby='none'></dialog>
        <button id='trigger'>run</button>
        <p id='result'></p>
        <script>
          const dialog = document.getElementById('dialog');
          document.getElementById('trigger').addEventListener('click', () => {
            const before = dialog.closedBy;
            dialog.closedBy = 'any';
            document.getElementById('result').textContent =
              before + ':' + dialog.closedBy + ':' + dialog.getAttribute('closedby');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#trigger")?;
    h.assert_text("#result", "none:any:any")?;
    Ok(())
}

#[test]
fn element_matches_method_works() -> Result<()> {
    let html = r#"
        <div id='container'>
          <button id='target' class='item primary'></button>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const direct = document.getElementById('target').matches('#target.item');
            const byTag = document.getElementById('target').matches('button');
            const bySelectorMismatch = document.getElementById('target').matches('.secondary');
            document.getElementById('result').textContent =
              direct + ':' + byTag + ':' + bySelectorMismatch;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:true:false")?;
    Ok(())
}

#[test]
fn element_closest_method_works() -> Result<()> {
    let html = r#"
        <div id='root'>
          <section id='scope'>
            <div id='container'>
              <button id='btn'>run</button>
            </div>
          </section>
        </div>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const scoped = document.getElementById('btn').closest('section');
            const selfMatch = document.getElementById('btn').closest('#btn');
            document.getElementById('result').textContent =
              scoped.id + ':' + selfMatch.id;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "scope:btn")?;
    Ok(())
}

#[test]
fn element_closest_method_returns_null_when_not_found() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const matched = document.getElementById('btn').closest('section');
            document.getElementById('result').textContent = matched ? 'found' : 'missing';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "missing")?;
    Ok(())
}

#[test]
fn query_selector_all_foreach_and_element_variables_work() -> Result<()> {
    let html = r#"
        <ul>
          <li class='item'>A</li>
          <li class='item'>B</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.querySelectorAll('.item').forEach((item, idx) => {
              item.setAttribute('data-idx', idx);
              item.classList.toggle('picked', idx === 1);
              document.getElementById('result').textContent =
                document.getElementById('result').textContent + item.textContent + item.getAttribute('data-idx');
            });
            document.getElementById('result').textContent =
              document.getElementById('result').textContent + ':' + document.querySelectorAll('.item')[1].classList.contains('picked');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "A0B1:true")?;
    Ok(())
}

#[test]
fn query_selector_all_foreach_single_arg_callback_works() -> Result<()> {
    let html = r#"
        <ul>
          <li class='item'>A</li>
          <li class='item'>B</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
        document.querySelectorAll('.item').forEach(item => {
              item.classList.add('seen');
              document.getElementById('result').textContent =
                document.getElementById('result').textContent + item.textContent;
            });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "AB")?;
    Ok(())
}

#[test]
fn parse_for_each_callback_accepts_arrow_expression_body() -> Result<()> {
    let (item_var, index_var, body) = parse_for_each_callback("item => 1")?;
    assert_eq!(item_var, "item");
    assert!(index_var.is_none());
    assert_eq!(body.len(), 1);
    match body
        .first()
        .expect("callback body should include one statement")
    {
        Stmt::Expr(Expr::Number(value)) => assert_eq!(*value, 1),
        other => panic!("unexpected callback body stmt: {other:?}"),
    }
    Ok(())
}

#[test]
fn listener_arrow_expression_callback_body_executes() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click',
            () => 1
          );
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.flush()?;
    h.assert_text("#result", "")?;
    Ok(())
}

#[test]
fn for_of_loop_supports_query_selector_all() -> Result<()> {
    let html = r#"
        <ul>
          <li class='item'>A</li>
          <li class='item'>B</li>
          <li class='item'>C</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let output = '';
            for (const item of document.querySelectorAll('.item')) {
              output = output + item.textContent;
            }
            document.getElementById('result').textContent = output;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "ABC")?;
    Ok(())
}

#[test]
fn for_in_loop_supports_query_selector_all_indexes() -> Result<()> {
    let html = r#"
        <ul>
          <li class='item'>A</li>
          <li class='item'>B</li>
          <li class='item'>C</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let output = '';
            for (let index in document.querySelectorAll('.item')) {
              output = output + index + ',';
            }
            document.getElementById('result').textContent = output;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "0,1,2,")?;
    Ok(())
}

#[test]
fn for_loop_supports_break_and_continue() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let out = '';
            for (let i = 0; i < 5; i = i + 1) {
              if (i === 0) {
                continue;
              }
              if (i === 3) {
                break;
              }
              out = out + i;
            }
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "12")?;
    Ok(())
}

#[test]
fn while_loop_supports_break_and_continue() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let out = '';
            let i = 0;
            while (i < 5) {
              i = i + 1;
              if (i === 1) {
                continue;
              }
              if (i === 4) {
                break;
              }
              out = out + i;
            }
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "23")?;
    Ok(())
}

#[test]
fn do_while_executes_at_least_once() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let count = 0;
            do {
              count = count + 1;
            } while (false);
            document.getElementById('result').textContent = count;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "1")?;
    Ok(())
}

#[test]
fn do_while_loop_supports_break_and_continue() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let i = 0;
            let out = '';
            do {
              i = i + 1;
              if (i === 1) {
                continue;
              }
              if (i === 4) {
                break;
              }
              out = out + i;
            } while (i < 5);
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "23")?;
    Ok(())
}

#[test]
fn document_scroll_event_fires_from_scroll_methods() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <div id='target'></div>
        <p id='result'></p>
        <script>
          const out = document.getElementById('result');
          document.addEventListener('scroll', (event) => {
            out.textContent = out.textContent + event.type.charAt(0);
          });

          document.getElementById('run').addEventListener('click', () => {
            const target = document.getElementById('target');
            target.scrollIntoView();
            target.scroll();
            target.scrollTo(1, 2);
            target.scrollBy({ top: 4 });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "ssss")?;
    Ok(())
}

#[test]
fn document_onscroll_property_assignment_works() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <div id='target'></div>
        <p id='result'></p>
        <script>
          let calls = 0;
          document.onscroll = (event) => {
            calls = calls + 1;
            document.getElementById('result').textContent = calls + ':' + event.type;
          };

          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('target').scrollIntoView();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "1:scroll")?;
    Ok(())
}

#[test]
fn document_onscroll_assignment_via_alias_works() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <div id='target'></div>
        <p id='result'></p>
        <script>
          const out = document.getElementById('result');
          const doc = document;
          const before = doc.onscroll === null;
          doc.onscroll = (event) => {
            out.textContent = out.textContent + event.type.charAt(0);
          };
          const after = typeof doc.onscroll === 'function';

          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('target').scroll();
            out.textContent = before + ':' + after + ':' + out.textContent;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:true:s")?;
    Ok(())
}

#[test]
fn document_scrollend_event_fires_after_scroll_when_position_changes() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <div id='target'></div>
        <p id='result'></p>
        <script>
          const out = document.getElementById('result');
          document.addEventListener('scroll', () => {
            out.textContent = out.textContent + 's';
          });
          document.addEventListener('scrollend', (event) => {
            out.textContent = out.textContent + (event.type === 'scrollend' ? 'e' : '?');
          });

          document.getElementById('run').addEventListener('click', () => {
            const target = document.getElementById('target');
            target.scrollTo(1, 2);
            target.scrollBy({ top: 4 });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "sese")?;
    Ok(())
}

#[test]
fn document_onscrollend_property_assignment_works() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <div id='target'></div>
        <p id='result'></p>
        <script>
          let calls = 0;
          document.onscrollend = (event) => {
            calls = calls + 1;
            document.getElementById('result').textContent =
              calls + ':' + event.type + ':' + event.cancelable;
          };

          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('target').scrollBy(0, 5);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "1:scrollend:false")?;
    Ok(())
}

#[test]
fn document_scrollend_does_not_fire_when_scroll_position_does_not_change() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <div id='target'></div>
        <p id='result'></p>
        <script>
          let calls = 0;
          document.addEventListener('scrollend', () => {
            calls = calls + 1;
          });

          document.getElementById('run').addEventListener('click', () => {
            const target = document.getElementById('target');
            target.scrollTo(0, 0);
            target.scrollBy(0, 0);
            target.scrollBy({ left: 0, top: 0 });
            document.getElementById('result').textContent = String(calls);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "0")?;
    Ok(())
}
