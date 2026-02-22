use super::*;

#[test]
fn u_has_implicit_generic_role_and_annotation_usage_work() -> Result<()> {
    let html = r#"
        <p>
          You could use this element to highlight
          <u id='spell' class='spelling'>speling</u> mistakes, so the writer can
          <u id='correct'>corect</u> them.
        </p>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const spell = document.getElementById('spell');
            const correct = document.getElementById('correct');
            document.getElementById('result').textContent =
              spell.role + ':' +
              correct.role + ':' +
              spell.className + ':' +
              document.querySelectorAll('u').length + ':' +
              spell.textContent.trim() + ':' +
              correct.textContent.trim() + ':' +
              spell.tagName;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "generic:generic:spelling:2:speling:corect:U")?;
    Ok(())
}

#[test]
fn u_role_override_and_remove_restores_implicit_generic_role() -> Result<()> {
    let html = r#"
        <p>
          <u id='note' title='Potential misspelling'>wrnogly</u>
        </p>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const note = document.getElementById('note');
            const initial = note.role + ':' + note.getAttribute('title');

            note.role = 'note';
            const assigned = note.role + ':' + note.getAttribute('role');

            note.removeAttribute('role');
            const restored = note.role + ':' + (note.getAttribute('role') === null);

            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "generic:Potential misspelling|note:note|generic:true",
    )?;
    Ok(())
}
