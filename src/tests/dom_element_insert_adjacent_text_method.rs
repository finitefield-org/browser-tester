use super::*;

#[test]
fn element_insert_adjacent_text_inserts_at_all_positions_and_returns_undefined() -> Result<()> {
    let html = r#"
        <div id='root'><span id='b'>B</span></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const root = document.getElementById('root');
            const b = document.getElementById('b');

            const r1 = b.insertAdjacentText('beforebegin', 'A') === undefined;
            const r2 = b.insertAdjacentText('afterbegin', 'X') === undefined;
            const r3 = b.insertAdjacentText('beforeend', 'Y') === undefined;
            const r4 = b.insertAdjacentText('afterend', 'C') === undefined;

            document.getElementById('result').textContent = [
              r1,
              r2,
              r3,
              r4,
              root.textContent,
              b.textContent,
              b.childNodes.length
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:true:true:true:AXBYC:XBY:3")?;
    Ok(())
}

#[test]
fn element_insert_adjacent_text_invalid_position_throws_syntax_error() -> Result<()> {
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
              const returned = b.insertAdjacentText(pos, 'Q');
              document.getElementById('result').setAttribute('data-returned', String(returned));
            } catch (e) {
              syntaxError = String(e).includes('SyntaxError');
            }

            document.getElementById('result').textContent = [
              syntaxError,
              document.getElementById('root').textContent,
              b.childNodes.length
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:B:1")?;
    Ok(())
}

#[test]
fn element_insert_adjacent_text_beforebegin_and_afterend_are_noop_without_element_parent(
) -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const detached = document.createElement('span');
            detached.textContent = 'T';
            const detachedBefore = detached.insertAdjacentText('beforebegin', 'L');
            const detachedAfter = detached.insertAdjacentText('afterend', 'R');

            const fragment = document.createDocumentFragment();
            const inner = document.createElement('span');
            inner.textContent = 'I';
            fragment.appendChild(inner);
            const fragmentBefore = inner.insertAdjacentText('beforebegin', 'A');
            const fragmentAfter = inner.insertAdjacentText('afterend', 'B');

            document.getElementById('result').textContent = [
              detachedBefore === undefined,
              detachedAfter === undefined,
              fragmentBefore === undefined,
              fragmentAfter === undefined,
              detached.textContent,
              fragment.childNodes.length,
              fragment.textContent,
              inner.textContent
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:true:true:true:T:1:I:I")?;
    Ok(())
}

#[test]
fn element_insert_adjacent_text_rejects_wrong_argument_count() -> Result<()> {
    let html = r#"
        <div id='root'><span id='b'>B</span></div>
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const b = document.getElementById('b');
            const returned = b.insertAdjacentText('beforeend');
            document.body.setAttribute('data-returned', String(returned));
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("insertAdjacentText requires exactly two arguments"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}
