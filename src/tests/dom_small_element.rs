use super::*;

#[test]
fn small_has_implicit_generic_role_and_side_comment_content_work() -> Result<()> {
    let html = r#"
        <p>
          MDN Web Docs is a learning platform for Web technologies and the software
          that powers the Web.
        </p>
        <p>
          <small id='legal'>
            The content is licensed under a Creative Commons license.
          </small>
        </p>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const legal = document.getElementById('legal');
            document.getElementById('result').textContent =
              legal.role + ':' +
              legal.tagName + ':' +
              legal.textContent.includes('Creative Commons') + ':' +
              legal.textContent.replace(/\s+/g, ' ').trim() + ':' +
              document.querySelectorAll('small').length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "generic:SMALL:true:The content is licensed under a Creative Commons license.:1",
    )?;
    Ok(())
}

#[test]
fn small_role_override_and_restore_roundtrip_work() -> Result<()> {
    let html = r#"
        <p>
          <small id='note'>Terms and conditions apply.</small>
        </p>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const note = document.getElementById('note');
            const initial = note.role + ':' + note.textContent.trim();

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
        "generic:Terms and conditions apply.|note:note|generic:true",
    )?;
    Ok(())
}
