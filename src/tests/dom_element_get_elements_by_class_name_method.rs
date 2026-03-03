use super::*;

#[test]
fn element_get_elements_by_class_name_returns_live_descendant_collection() -> Result<()> {
    let html = r#"
        <div id='root' class='item hot'>
          <p id='a' class='item hot'>A</p>
          <p id='b' class='item'>B</p>
          <section id='host'><p id='c' class='item hot'>C</p></section>
        </div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const root = document.getElementById('root');
            const list = root.getElementsByClassName('item hot');

            const before = list.length + ':' + list[0].id + ':' + list[1].id;

            document.getElementById('c').className = 'item';
            const afterRemove = list.length + ':' + list[0].id;

            const d = document.createElement('p');
            d.id = 'd';
            d.className = 'item hot';
            document.getElementById('host').appendChild(d);
            const afterAdd = list.length + ':' + list[1].id;

            document.getElementById('result').textContent =
              before + '|' + afterRemove + '|' + afterAdd;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "2:a:c|1:a|2:d")?;
    Ok(())
}

#[test]
fn element_get_elements_by_class_name_accepts_whitespace_and_empty_string() -> Result<()> {
    let html = r#"
        <div id='root'>
          <span id='x' class='test red'>X</span>
          <span id='y' class='test'>Y</span>
        </div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const root = document.getElementById('root');
            const spaced = root.getElementsByClassName('  test   red  ');
            const empty = root.getElementsByClassName('');
            document.getElementById('result').textContent = [
              spaced.length,
              spaced[0].id,
              empty.length,
              empty[0] === undefined
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "1:x:0:true")?;
    Ok(())
}

#[test]
fn element_get_elements_by_class_name_rejects_wrong_argument_count() -> Result<()> {
    let html = r#"
        <div id='root'></div>
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('root').getElementsByClassName();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("getElementsByClassName requires exactly one argument"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}
