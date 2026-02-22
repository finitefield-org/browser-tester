use super::*;

#[test]
fn bdo_implicit_generic_role_and_dir_reflection_work() -> Result<()> {
    let html = r#"
        <p>
          In the computer's memory:
          <bdo id='memory' class='sample' dir='ltr'>אה, אני אוהב להיות ליד חוף הים</bdo>
        </p>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const memory = document.getElementById('memory');
            document.getElementById('result').textContent =
              memory.role + ':' +
              memory.dir + ':' +
              memory.getAttribute('dir') + ':' +
              document.querySelectorAll('bdo.sample').length + ':' +
              memory.tagName;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "generic:ltr:ltr:1:BDO")?;
    Ok(())
}

#[test]
fn bdo_dir_and_role_assignments_override_and_restore() -> Result<()> {
    let html = r#"
        <p><bdo id='switch' dir='rtl'>This text goes right to left.</bdo></p>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const switchEl = document.getElementById('switch');

            const initialDir = switchEl.dir;
            switchEl.dir = 'ltr';
            const assignedDir = switchEl.dir + ':' + switchEl.getAttribute('dir');
            switchEl.removeAttribute('dir');
            const restoredDir = switchEl.dir + ':' + (switchEl.getAttribute('dir') === null);

            const initialRole = switchEl.role;
            switchEl.role = 'note';
            const assignedRole = switchEl.role + ':' + switchEl.getAttribute('role');
            switchEl.removeAttribute('role');
            const restoredRole = switchEl.role + ':' + (switchEl.getAttribute('role') === null);

            document.getElementById('result').textContent =
              initialDir + '|' +
              assignedDir + '|' +
              restoredDir + '|' +
              initialRole + '|' +
              assignedRole + '|' +
              restoredRole;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "rtl|ltr:ltr|:true|generic|note:note|generic:true",
    )?;
    Ok(())
}
