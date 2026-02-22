use super::*;

#[test]
fn i_implicit_generic_role_and_idiomatic_usage_work() -> Result<()> {
    let html = r#"
        <p>
          The phrase <i id='latin' lang='la'>Veni, vidi, vici</i> and the term
          <i id='term'>bandwidth</i> are both set off from normal prose.
        </p>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const latin = document.getElementById('latin');
            const term = document.getElementById('term');
            document.getElementById('result').textContent =
              latin.role + ':' +
              term.role + ':' +
              latin.getAttribute('lang') + ':' +
              document.querySelectorAll('i').length + ':' +
              term.textContent.trim() + ':' +
              latin.tagName;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "generic:generic:la:2:bandwidth:I")?;
    Ok(())
}

#[test]
fn i_role_override_and_remove_restores_implicit_generic_role() -> Result<()> {
    let html = r#"
        <p>I looked at it and thought <i id='thought'>This cannot be real!</i></p>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const thought = document.getElementById('thought');
            const initial = thought.role;
            thought.role = 'note';
            const assigned = thought.role + ':' + thought.getAttribute('role');
            thought.removeAttribute('role');
            const restored = thought.role + ':' + (thought.getAttribute('role') === null);
            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "generic|note:note|generic:true")?;
    Ok(())
}
