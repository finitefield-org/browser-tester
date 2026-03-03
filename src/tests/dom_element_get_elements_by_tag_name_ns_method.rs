use super::*;

#[test]
fn element_get_elements_by_tag_name_ns_filters_and_is_live() -> Result<()> {
    let html = r#"
        <div id='root'><div id='host'></div></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const root = document.getElementById('root');
            const host = document.getElementById('host');

            const a = document.createElementNS('http://example.com/ns', 'x:item');
            a.id = 'a';
            const b = document.createElementNS('http://example.com/ns', 'item');
            b.id = 'b';
            const c = document.createElement('item');
            c.id = 'c';

            host.appendChild(a);
            host.appendChild(b);
            host.appendChild(c);

            const list = root.getElementsByTagNameNS('http://example.com/ns', 'item');
            const before = list.length + ':' + list[0].id + ':' + list[1].id;

            b.remove();
            const afterRemove = list.length + ':' + list[0].id;

            const d = document.createElementNS('http://example.com/ns', 'item');
            d.id = 'd';
            host.appendChild(d);
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
fn element_get_elements_by_tag_name_ns_supports_wildcards_and_null_namespace() -> Result<()> {
    let html = r#"
        <div id='root'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const root = document.getElementById('root');

            const a = document.createElementNS('http://example.com/ns', 'x:item');
            a.id = 'a';
            const b = document.createElementNS('http://example.com/ns', 'x:box');
            b.id = 'b';
            const c = document.createElement('item');
            c.id = 'c';
            const d = document.createElementNS(null, 'item');
            d.id = 'd';

            root.appendChild(a);
            root.appendChild(b);
            root.appendChild(c);
            root.appendChild(d);

            const anyNsItem = root.getElementsByTagNameNS('*', 'item');
            const nullNsItem = root.getElementsByTagNameNS(null, 'item');
            const emptyNsItem = root.getElementsByTagNameNS('', 'item');
            const nsAnyLocal = root.getElementsByTagNameNS('http://example.com/ns', '*');

            document.getElementById('result').textContent = [
              anyNsItem.length,
              anyNsItem[0].id,
              anyNsItem[1].id,
              anyNsItem[2].id,
              nullNsItem.length,
              nullNsItem[0].id,
              emptyNsItem.length,
              emptyNsItem[0].id,
              nsAnyLocal.length,
              nsAnyLocal[0].id,
              nsAnyLocal[1].id
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "3:a:c:d:1:d:1:d:2:a:b")?;
    Ok(())
}

#[test]
fn element_get_elements_by_tag_name_ns_rejects_wrong_argument_count() -> Result<()> {
    let html = r#"
        <div id='root'></div>
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('root').getElementsByTagNameNS('http://example.com/ns');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("getElementsByTagNameNS requires exactly two arguments"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}
