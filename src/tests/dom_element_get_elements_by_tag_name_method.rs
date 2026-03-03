use super::*;

#[test]
fn element_get_elements_by_tag_name_is_live_and_excludes_self() -> Result<()> {
    let html = r#"
        <section id='root'>
          <p id='a'>A</p>
          <div id='host'><p id='b'>B</p></div>
        </section>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const root = document.getElementById('root');
            const list = root.getElementsByTagName('P');
            const before = list.length + ':' + list[0].id + ':' + list[1].id;

            document.getElementById('b').remove();
            const afterRemove = list.length + ':' + list[0].id;

            const c = document.createElement('P');
            c.id = 'c';
            document.getElementById('host').appendChild(c);
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
fn element_get_elements_by_tag_name_wildcard_returns_descendants_in_tree_order() -> Result<()> {
    let html = r#"
        <div id='root'>
          <section id='s1'><p id='p1'></p></section>
          <section id='s2'><p id='p2'></p><span id='x'></span></section>
        </div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const all = document.getElementById('root').getElementsByTagName('*');
            document.getElementById('result').textContent = [
              all.length,
              all[0].id,
              all[1].id,
              all[2].id,
              all[3].id,
              all[4].id
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "5:s1:p1:s2:p2:x")?;
    Ok(())
}

#[test]
fn element_get_elements_by_tag_name_rejects_wrong_argument_count() -> Result<()> {
    let html = r#"
        <div id='root'><p></p></div>
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('root').getElementsByTagName();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("getElementsByTagName requires exactly one argument"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}
