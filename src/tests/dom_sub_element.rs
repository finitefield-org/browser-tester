use super::*;

#[test]
fn sub_has_implicit_subscript_role_and_supports_chemical_formula_content() -> Result<()> {
    let html = r#"
        <p id='formula'>
          C<sub id='c8'>8</sub>H<sub>10</sub>N<sub>4</sub>O<sub>2</sub>
        </p>
        <p id='note'>
          Nakamura, Johnson, and Mason<sub id='footnote'>1</sub>
        </p>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const c8 = document.getElementById('c8');
            const footnote = document.getElementById('footnote');

            document.getElementById('result').textContent =
              c8.role + ':' +
              footnote.role + ':' +
              c8.tagName + ':' +
              document.querySelectorAll('sub').length + ':' +
              document.getElementById('formula').textContent.replace(/\s+/g, '') + ':' +
              footnote.textContent.trim();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "subscript:subscript:SUB:5:C8H10N4O2:1")?;
    Ok(())
}

#[test]
fn sub_role_override_and_remove_restores_implicit_subscript_role() -> Result<()> {
    let html = r#"
        <p>
          x<sub id='index'>1</sub> ... x<sub>n</sub>
        </p>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const index = document.getElementById('index');
            const initial = index.role + ':' + index.textContent.trim();

            index.role = 'note';
            const assigned = index.role + ':' + index.getAttribute('role');

            index.removeAttribute('role');
            const restored = index.role + ':' + (index.getAttribute('role') === null);

            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "subscript:1|note:note|subscript:true")?;
    Ok(())
}
