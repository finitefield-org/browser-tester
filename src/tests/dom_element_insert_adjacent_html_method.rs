use super::*;

#[test]
fn element_insert_adjacent_html_inserts_at_all_positions_and_returns_undefined() -> Result<()> {
    let html = r#"
        <div id='root'><span id='b'>B</span></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const root = document.getElementById('root');
            const b = document.getElementById('b');

            const r1 = b.insertAdjacentHTML('beforebegin', '<i id="a">A</i>') === undefined;
            const r2 = b.insertAdjacentHTML('AfTeRbEgIn', '<i id="d">D</i>') === undefined;
            const r3 = b.insertAdjacentHTML('beforeend', '<i id="e">E</i>') === undefined;
            const r4 = b.insertAdjacentHTML('afterend', '<i id="c">C</i>') === undefined;

            document.getElementById('result').textContent = [
              r1,
              r2,
              r3,
              r4,
              root.textContent,
              b.firstElementChild.id,
              b.lastElementChild.id
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:true:true:true:ADBEC:d:e")?;
    Ok(())
}

#[test]
fn element_insert_adjacent_html_invalid_position_throws_syntax_error() -> Result<()> {
    let html = r#"
        <div id='root'><span id='b'>B</span></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const b = document.getElementById('b');
            const pos = 'middle';
            let syntaxError = false;

            try {
              const returned = b.insertAdjacentHTML(pos, '<i id="x">X</i>');
              document.getElementById('result').setAttribute('data-returned', String(returned));
            } catch (e) {
              syntaxError = String(e).includes('SyntaxError');
            }

            document.getElementById('result').textContent = [
              syntaxError,
              document.querySelectorAll('#x').length,
              document.getElementById('root').textContent
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:0:B")?;
    Ok(())
}

#[test]
fn element_insert_adjacent_html_beforebegin_and_afterend_require_parent_element() -> Result<()> {
    let html = r#"
        <div id='top'>T</div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const top = document.getElementById('top');
            const detached = document.createElement('div');

            let detachedBefore = false;
            try {
              const returnedDetached = detached.insertAdjacentHTML('beforebegin', '<i id="det">D</i>');
              document.getElementById('result').setAttribute('data-detached', String(returnedDetached));
            } catch (e) {
              detachedBefore = String(e).includes('NoModificationAllowedError');
            }

            let docBefore = false;
            try {
              const returnedDocBefore = top.insertAdjacentHTML('beforebegin', '<i id="x1">X1</i>');
              document.getElementById('result').setAttribute('data-doc-before', String(returnedDocBefore));
            } catch (e) {
              docBefore = String(e).includes('NoModificationAllowedError');
            }

            let docAfter = false;
            try {
              const returnedDocAfter = top.insertAdjacentHTML('afterend', '<i id="x2">X2</i>');
              document.getElementById('result').setAttribute('data-doc-after', String(returnedDocAfter));
            } catch (e) {
              docAfter = String(e).includes('NoModificationAllowedError');
            }

            document.getElementById('result').textContent = [
              detachedBefore,
              docBefore,
              docAfter,
              document.querySelectorAll('#x1').length,
              document.querySelectorAll('#x2').length,
              document.querySelectorAll('#det').length,
              top.textContent
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:true:true:0:0:0:T")?;
    Ok(())
}

#[test]
fn element_insert_adjacent_html_rejects_wrong_argument_count() -> Result<()> {
    let html = r#"
        <div id='root'><span id='b'>B</span></div>
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const b = document.getElementById('b');
            const returned = b.insertAdjacentHTML('beforeend');
            document.body.setAttribute('data-returned', String(returned));
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("insertAdjacentHTML requires exactly two arguments"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}
