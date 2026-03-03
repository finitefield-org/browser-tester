use super::*;

#[test]
fn element_insert_adjacent_element_inserts_at_all_positions_and_returns_inserted_element(
) -> Result<()> {
    let html = r#"
        <div id='root'><span id='b'>B</span></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const root = document.getElementById('root');
            const b = document.getElementById('b');

            const a = document.createElement('span');
            a.id = 'a';
            a.textContent = 'A';
            const c = document.createElement('span');
            c.id = 'c';
            c.textContent = 'C';
            const d = document.createElement('span');
            d.id = 'd';
            d.textContent = 'D';
            const e = document.createElement('span');
            e.id = 'e';
            e.textContent = 'E';

            const r1 = b.insertAdjacentElement('BeFoReBeGiN', a) === a;
            const r2 = b.insertAdjacentElement('afterbegin', d) === d;
            const r3 = b.insertAdjacentElement('beforeend', e) === e;
            const r4 = b.insertAdjacentElement('afterend', c) === c;

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
fn element_insert_adjacent_element_invalid_position_throws_syntax_error() -> Result<()> {
    let html = r#"
        <div id='root'><span id='b'>B</span></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const b = document.getElementById('b');
            const x = document.createElement('span');
            x.id = 'x';
            x.textContent = 'X';
            const pos = 'middle';

            let syntaxError = false;
            try {
              const returned = b.insertAdjacentElement(pos, x);
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
fn element_insert_adjacent_element_non_element_argument_throws_type_error() -> Result<()> {
    let html = r#"
        <div id='root'><span id='b'>B</span></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const b = document.getElementById('b');
            const textNode = document.createTextNode('X');

            let textNodeTypeError = false;
            try {
              const returnedTextNode = b.insertAdjacentElement('beforeend', textNode);
              document.getElementById('result').setAttribute('data-text-node', String(returnedTextNode));
            } catch (e) {
              textNodeTypeError = String(e).includes('TypeError');
            }

            let primitiveTypeError = false;
            try {
              const returnedPrimitive = b.insertAdjacentElement('beforeend', 'X');
              document.getElementById('result').setAttribute('data-primitive', String(returnedPrimitive));
            } catch (e) {
              primitiveTypeError = String(e).includes('TypeError');
            }

            document.getElementById('result').textContent = [
              textNodeTypeError,
              primitiveTypeError,
              b.textContent
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:true:B")?;
    Ok(())
}

#[test]
fn element_insert_adjacent_element_returns_null_when_insertion_fails() -> Result<()> {
    let html = r#"
        <div id='a'><span id='b'>B</span></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const a = document.getElementById('a');
            const b = document.getElementById('b');

            const cycle = b.insertAdjacentElement('afterbegin', a);

            const detached = document.createElement('span');
            detached.textContent = 'T';
            const outsideLeft = detached.insertAdjacentElement('beforebegin', document.createElement('i'));
            const outsideRight = detached.insertAdjacentElement('afterend', document.createElement('em'));

            document.getElementById('result').textContent = [
              cycle === null,
              outsideLeft === null,
              outsideRight === null,
              a.textContent,
              b.textContent,
              a.contains(b)
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:true:true:B:B:true")?;
    Ok(())
}
